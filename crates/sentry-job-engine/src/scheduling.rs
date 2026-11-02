use crate::dependencies::JobDependencies;
use crate::error::JobEngineError;
use crate::job::Job;
use tokio::sync::{mpsc, oneshot, watch};
use tracing::{error, info, instrument, warn, Instrument, debug};

enum SchedulerMessage<'a> {
    Job(Box<dyn Job<'a>>),
    Shutdown(oneshot::Sender<()>),
}
#[derive(Clone, Debug)]
pub struct SchedulerHandle<'a> {
    sender: mpsc::Sender<SchedulerMessage<'a>>,
    shutdown_rx: watch::Receiver<bool>,
}
impl<'a> SchedulerHandle<'a> {
    #[instrument(skip(self, job))]
    pub async fn submit<J: Job<'a>>(&self, job: J) -> Result<(), JobEngineError> {
        let sub = self.sender
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
    async fn internal_shutdown(&self) -> oneshot::Receiver<()> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(SchedulerMessage::Shutdown(tx))
            .await
            .expect("Scheduler is already gone");
        debug!("Shutdown message placed on queue");
        rx
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
    dependencies: Option<JobDependencies>,
}

impl<'a> Scheduler<'a> {
    pub fn new(buffer: usize) -> (Self, SchedulerHandle<'a>) {
        let (sender, receiver) = mpsc::channel(buffer);
        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        let scheduler = Scheduler {
            receiver,
            shutdown_tx,
            dependencies: None,
        };
        let handle = SchedulerHandle {
            sender,
            shutdown_rx,
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
        let mut clean_lever = false;
        while let Some(message) = self.receiver.recv().await {
            match message {
                SchedulerMessage::Job(mut job) => {
                    if !clean_lever {
                        let job_name = job.job_type();
                        let mut user_id = "unknown".to_string();
                        if let Some(dependencies) = &self.dependencies {
                            let id = dependencies.configuration().user_id.clone();
                            user_id = id.to_string();
                        }
                        let span = tracing::info_span!("job-message", user_id = user_id, job_name = job_name);
                        job.inject_dependencies(self.dependencies.as_ref().unwrap());
                        span.follows_from(tracing::Span::current());
                        let job_result = job.execute(handle.clone()).instrument(span).await;
                        if job_result.is_err() {
                            let e = job_result.unwrap_err();
                            error!("Failed to execute job: {:?}", e);
                            warn!(
                                "A job in the queue has errored out queue will be alive until shutdown hook is called"
                            );
                            clean_lever = true;
                            // maybe?
                            // handle.clone().shutdown().await;
                        }
                    } else {
                        warn!("Scheduler ignoring job: {}", job.job_type());
                    }
                }
                SchedulerMessage::Shutdown(ack) => {
                    debug!("Shutdown hook triggered, closing queue");
                    self.receiver.close();
                    // TODO: this functionality never really gets activated.. would probably be a good
                    //       idea to make a "place last" functionality when queue reaches end-of-mission.
                    while let Ok(msg) = self.receiver.try_recv() {
                        if let SchedulerMessage::Job(job) = msg {
                            let job_name = job.job_type();
                            let span =
                                tracing::info_span!("job-message-cleanup", job_name = job_name);
                            span.follows_from(tracing::Span::current());
                            let job_result = job.execute(handle.clone()).instrument(span).await;
                            if job_result.is_err() {
                                let e = job_result.unwrap_err();
                                error!("Failed to execute job: {:?}", e);
                            }
                        }
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
    use crate::job::Job;
    use crate::scheduling::{Scheduler, SchedulerHandle};
    use async_trait::async_trait;
    use tracing::{info, instrument};

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

        #[instrument]
        async fn execute(&self, _: SchedulerHandle<'a>) -> Result<(), JobEngineError> {
            info!("Running job {}", self.id);
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            info!("Job {} done", self.id);
            Ok(())
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
            .expect("TODO: panic message");
        handle2
            .submit(PrintJob { id: 2 })
            .await
            .expect("TODO: panic message");
        info!("Verification printout");
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        handle.shutdown().await;
    }
}