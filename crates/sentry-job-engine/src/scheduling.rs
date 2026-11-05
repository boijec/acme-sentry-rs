use crate::dependencies::JobDependencies;
use crate::error::{ACMES132, JobEngineError};
use crate::job::{Job, JobExecutionFlag};
use std::ops::Deref;
use tokio::sync::oneshot::{Receiver, Sender};
use tokio::sync::{mpsc, oneshot, watch};
use tracing::{Instrument, debug, error, info, instrument};

enum SchedulerMessage<'a> {
    Job(Box<dyn Job<'a>>),
    Shutdown(Sender<()>),
}
#[derive(Clone, Debug)]
pub struct SchedulerHandle<'a> {
    sender: mpsc::Sender<SchedulerMessage<'a>>,
    shutdown_rx: watch::Receiver<bool>,
    report_rx: watch::Receiver<JobExecutionFlag>,
}
impl<'a> SchedulerHandle<'a> {
    #[instrument(skip(self, job))]
    pub async fn submit<J: Job<'a>>(&self, job: J) -> Result<(), JobEngineError> {
        let sub = self
            .sender
            .send(SchedulerMessage::Job(Box::new(job)))
            .await
            .map_err(|_| JobEngineError::SchedulingError("Scheduler is shutting down".into()));
        debug!("Message submitted to scheduler");
        sub
    }
    #[instrument(skip(self))]
    pub async fn shutdown(self) {
        debug!("Shutdown initiated");
        let rx = self.internal_shutdown().await;
        rx.await.expect("Scheduler didn't confirm shutdown");
    }
    #[instrument(skip(self))]
    async fn internal_shutdown(&self) -> Receiver<()> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(SchedulerMessage::Shutdown(tx))
            .await
            .expect("Scheduler is already gone");
        debug!("Shutdown message placed on queue");
        rx
    }
    pub async fn wait_for_completion(&mut self) {
        self.report_rx
            .wait_for(|status| {
                return match status {
                    JobExecutionFlag::Fatal(code) => {
                        error!("SCHEDULER CRASH: {}", code.explain());
                        true
                    }
                    JobExecutionFlag::Stop => {
                        info!("Job engine reached final stage");
                        true
                    }
                    _ => false,
                };
            })
            .await
            .expect("Scheduler didn't confirm completion");
        let rx = self.internal_shutdown().await;
        rx.await.expect("Scheduler didn't confirm shutdown");
    }
    pub async fn wait_for_healthy(&mut self) -> Result<(), JobEngineError> {
        let ref_flag = self
            .report_rx
            .wait_for(|status| {
                return match status {
                    JobExecutionFlag::PreflightCompleted => true,
                    JobExecutionFlag::Fatal(_) => true,
                    _ => false,
                };
            })
            .await
            .map_err(|_| {
                JobEngineError::JobExecutionError(
                    "Error while waiting for preflight-confirmation".into(),
                )
            })?;
        let flag = ref_flag.deref();
        match flag {
            JobExecutionFlag::Fatal(e) => Err(JobEngineError::JobExecutionError(e.explain())),
            _ => Ok(()),
        }
    }
    pub async fn wait_for_shutdown(&mut self) {
        self.shutdown_rx
            .wait_for(|v| *v)
            .await
            .expect("Scheduler dropped without signaling");
    }
}

pub struct Scheduler<'a> {
    receiver: mpsc::Receiver<SchedulerMessage<'a>>,
    shutdown_tx: watch::Sender<bool>,
    report_tx: watch::Sender<JobExecutionFlag>,
    dependencies: Option<JobDependencies>,
}

impl<'a> Scheduler<'a> {
    pub fn new(buffer: usize) -> (Self, SchedulerHandle<'a>) {
        let (sender, receiver) = mpsc::channel(buffer);
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let (report_tx, report_rx) = watch::channel(JobExecutionFlag::Next);

        let scheduler = Scheduler {
            receiver,
            shutdown_tx,
            report_tx,
            dependencies: None,
        };
        let handle = SchedulerHandle {
            sender,
            shutdown_rx,
            report_rx,
        };
        (scheduler, handle)
    }
    pub fn with_dependencies(&mut self, job_dependencies: JobDependencies) {
        self.dependencies = Some(job_dependencies);
    }
    pub async fn run(mut self, handle: SchedulerHandle<'a>) {
        if self.dependencies.as_ref().is_none() {
            error!("No job dependencies declared for job scheduler, shutting down..");
            let _ = self.shutdown_tx.send(true);
            return;
        }
        info!("Scheduler started");
        while let Some(message) = self.receiver.recv().await {
            match message {
                SchedulerMessage::Job(mut job) => {
                    let job_name = job.job_type();
                    let mut user_id = "unknown".to_string();
                    if let Some(dependencies) = &self.dependencies {
                        let id = dependencies.configuration().user_id.clone();
                        user_id = id.to_string();
                    }
                    let span =
                        tracing::info_span!("job-message", user_id = user_id, job_name = job_name);
                    job.inject_dependencies(self.dependencies.as_ref().unwrap());
                    span.follows_from(tracing::Span::current());
                    let job_result = job.execute(handle.clone()).instrument(span).await;
                    if let Ok(t) = job_result {
                        let _ = self.report_tx.send(t);
                    } else if let Err(job_result) = job_result {
                        error!(
                            "Failed to execute step '{}' with error :: {}",
                            job_name, job_result
                        );
                        if let JobEngineError::FlagError(e) = job_result {
                            let _ = self.report_tx.send(e);
                        } else {
                            let _ = self.report_tx.send(ACMES132);
                        }
                    }
                }
                SchedulerMessage::Shutdown(ack) => {
                    debug!("Shutdown hook triggered, closing queue");
                    self.receiver.close();
                    while let Ok(msg) = self.receiver.try_recv() {
                        drop(msg);
                        debug!("Scheduler draining queue");
                    }
                    let _ = ack.send(());
                    debug!("Scheduler shutdown ack sent");
                    break;
                }
            }
        }
        let _ = self.shutdown_tx.send(true);
        info!("Scheduler stopped");
    }
}

#[cfg(test)]
mod tests {
    use crate::dependencies::JobDependencies;
    use crate::error::JobEngineError;
    use crate::job::{Job, JobExecutionFlag};
    use crate::scheduling::{Scheduler, SchedulerHandle};
    use async_trait::async_trait;
    use tracing::instrument;

    #[derive(Debug)]
    struct PrintJob {
        id: u32,
    }
    #[async_trait]
    impl<'a> Job<'a> for PrintJob {
        fn job_type(&self) -> &'a str {
            "print-job"
        }

        fn inject_dependencies(&mut self, _dependencies: &JobDependencies) {
            return;
        }

        #[instrument(skip(self, _handle))]
        async fn execute(
            &self,
            _handle: SchedulerHandle<'a>,
        ) -> Result<JobExecutionFlag, JobEngineError> {
            println!("Running job {}", self.id);
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            println!("Job {} done", self.id);
            Ok(JobExecutionFlag::Next)
        }
    }

    #[tokio::test]
    async fn test_struct_queue() {
        let (scheduler, handle) = Scheduler::new(32);
        tokio::spawn(scheduler.run(handle.clone()));
        let handle2 = handle.clone();
        handle
            .submit(PrintJob { id: 1 })
            .await
            .expect("panic message");
        handle2
            .submit(PrintJob { id: 2 })
            .await
            .expect("panic message");
        println!("Verification printout");
        handle.clone().wait_for_shutdown().await;
    }
}
