use crate::error::{ACMES132, JobEngineError};
use crate::job::{Job, JobContext, JobExecutionFlag, JobKind};
use std::future::poll_fn;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{Semaphore, mpsc, watch};
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tokio_util::time::DelayQueue;
use tracing::{Instrument, Span, debug, error, info, instrument};

fn build_command<D, J: JobKind<D>>(job: J, at: Option<SystemTime>) -> Command<D> {
    let delay = at
        .map(|t| {
            t.duration_since(SystemTime::now())
                .unwrap_or(Duration::ZERO)
        })
        .unwrap_or(Duration::ZERO);
    Command::Submit {
        job: Box::new(job),
        run_at: Instant::now() + delay,
    }
}

fn closed() -> JobEngineError {
    JobEngineError::SchedulingError("Scheduler is shutting down".into())
}

enum Command<D> {
    Submit {
        job: Box<dyn Job<D>>,
        run_at: Instant,
    },
}
#[derive(Clone)]
pub struct JobHandle<D> {
    sender: mpsc::WeakSender<Command<D>>,
    shutdown: CancellationToken,
}

impl<D> JobHandle<D>
where
    D: Clone + Sync + Send + 'static,
{
    pub async fn submit<J: JobKind<D>>(
        &self,
        job: J,
        at: Option<SystemTime>,
    ) -> Result<(), JobEngineError> {
        let tx = self.sender.upgrade().ok_or_else(closed)?;
        tx.send(build_command(job, at)).await.map_err(|_| closed())
    }

    pub fn is_shutting_down(&self) -> bool {
        self.shutdown.is_cancelled()
    }

    pub async fn cancelled(&self) {
        self.shutdown.cancelled().await
    }
}

#[derive(Clone)]
pub struct SchedulerHandle<D> {
    sender: mpsc::Sender<Command<D>>,
    shutdown: CancellationToken,
    stopped: CancellationToken,
    report_rx: watch::Receiver<JobExecutionFlag>,
}

impl<D> SchedulerHandle<D>
where
    D: Clone + Send + 'static,
{
    #[instrument(skip(self, job))]
    pub async fn submit<J: JobKind<D>>(
        &self,
        job: J,
        at: Option<SystemTime>,
    ) -> Result<(), JobEngineError> {
        self.sender
            .send(build_command(job, at))
            .await
            .map_err(|_| closed())?;
        debug!("Message submitted to scheduler");
        Ok(())
    }

    pub(crate) fn job_handle(&self) -> JobHandle<D> {
        JobHandle::<D> {
            sender: self.sender.downgrade(),
            shutdown: self.shutdown.clone(),
        }
    }

    pub async fn shutdown(self) {
        debug!("Shutdown initiated");
        self.shutdown.cancel();
        self.stopped.cancelled().await;
    }

    pub async fn wait_for_shutdown(&self) {
        self.stopped.cancelled().await;
    }

    pub async fn wait_for_completion(&self) -> Result<(), JobEngineError> {
        let mut rx = self.report_rx.clone();
        let fatal = {
            let flag = rx
                .wait_for(|s| matches!(s, JobExecutionFlag::Fatal(_) | JobExecutionFlag::Stop))
                .await
                .map_err(|_| {
                    JobEngineError::JobExecutionError("Scheduler dropped before completion".into())
                })?;
            match &*flag {
                JobExecutionFlag::Fatal(code) => {
                    error!("SCHEDULER CRASH: {}", code.explain());
                    Some(code.explain())
                }
                _ => {
                    info!("Job engine reached final stage");
                    None
                }
            }
        };
        self.shutdown.cancel();
        self.stopped.cancelled().await;
        fatal.map_or(Ok(()), |e| Err(JobEngineError::JobExecutionError(e)))
    }

    pub async fn wait_for_healthy(&self) -> Result<(), JobEngineError> {
        let mut rx = self.report_rx.clone();
        let flag = rx
            .wait_for(|s| {
                matches!(
                    s,
                    JobExecutionFlag::PreflightCompleted | JobExecutionFlag::Fatal(_)
                )
            })
            .await
            .map_err(|_| {
                JobEngineError::JobExecutionError(
                    "Error while waiting for preflight-confirmation".into(),
                )
            })?;
        match &*flag {
            JobExecutionFlag::Fatal(e) => Err(JobEngineError::JobExecutionError(e.explain())),
            _ => Ok(()),
        }
    }
}

pub struct Scheduler<D> {
    receiver: mpsc::Receiver<Command<D>>,
    shutdown: CancellationToken,
    stopped: CancellationToken,
    report_tx: watch::Sender<JobExecutionFlag>,
    job_handle: JobHandle<D>,
    dependencies: Option<D>,
    label: String,
    max_concurrency: usize,
}

impl<D> Scheduler<D>
where
    D: Clone + Sync + Send + 'static,
{
    pub fn new(label: impl Into<String>, capacity: usize) -> (Self, SchedulerHandle<D>) {
        let (tx, rx) = mpsc::channel(capacity);
        let shutdown = CancellationToken::new();
        let stopped = CancellationToken::new();
        let (report_tx, report_rx) = watch::channel(JobExecutionFlag::default());

        let handle = SchedulerHandle {
            sender: tx,
            shutdown: shutdown.clone(),
            stopped: stopped.clone(),
            report_rx,
        };
        let job_handle = handle.job_handle();
        let scheduler = Scheduler {
            receiver: rx,
            shutdown,
            stopped,
            report_tx,
            job_handle,
            dependencies: None,
            label: label.into(),
            max_concurrency: 3,
        };
        (scheduler, handle)
    }
    pub fn with_dependencies(&mut self, job_dependencies: D) {
        self.dependencies = Some(job_dependencies);
    }
    pub async fn run(mut self) {
        let Some(deps) = self.dependencies.take().map(Arc::new) else {
            error!("No job dependencies declared for job scheduler, shutting down..");
            self.shutdown.cancel();
            self.stopped.cancel();
            return;
        };
        info!("Scheduler started");

        let permits = Arc::new(Semaphore::new(self.max_concurrency));
        let tracker = TaskTracker::new();
        let mut queue: DelayQueue<Box<dyn Job<D>>> = DelayQueue::new();

        loop {
            tokio::select! {
                _ = self.shutdown.cancelled() => break,
                msg = self.receiver.recv() => match msg {
                    Some(Command::Submit { job, run_at }) => {
                        queue.insert_at(job, run_at.into_std().into());
                    }
                    None => break,
                },
                Some(expired) = poll_fn(|cx| queue.poll_expired(cx)), if !queue.is_empty() => {
                    let job = expired.into_inner();
                    let ctx = JobContext {
                        handle: self.job_handle.clone(),
                        deps: deps.clone(),
                        token: self.shutdown.child_token(),
                    };
                    let report_tx = self.report_tx.clone();
                    let permits = permits.clone();
                    let name = job.job_type().to_owned();
                    let span = tracing::info_span!(
                        "job-task", user_id = %self.label, task_name = %name
                    );
                    span.follows_from(Span::current());

                    tracker.spawn(
                        async move {
                            let Ok(_permit) = permits.acquire_owned().await else { return };
                            let span = tracing::info_span!("job");
                            span.follows_from(Span::current());
                            let outcome = tokio::spawn(job.execute(ctx).instrument(span)).await;
                            let flag = match outcome {
                                Ok(Ok(flag)) => flag,
                                Ok(Err(JobEngineError::FlagError(e))) => {
                                    error!("step '{name}' failed: {e:?}");
                                    e
                                }
                                Ok(Err(e)) => {
                                    error!("Failed to execute step '{name}' with error :: {e}");
                                    ACMES132
                                }
                                Err(join_err) => {
                                    error!("step '{name}' panicked or was aborted :: {join_err}");
                                    ACMES132
                                }
                            };
                            let _ = report_tx.send(flag);
                        }
                        .instrument(span),
                    );
                }
            }
        }
        debug!("Shutdown triggered, closing queue");
        self.shutdown.cancel();
        self.receiver.close();
        drop(queue);
        tracker.close();
        tracker.wait().await;
        self.stopped.cancel();
        info!("Scheduler stopped");
    }
}
