use async_trait::async_trait;
use serde_json::Value;
use tokio::sync::{mpsc, oneshot, watch};
use tracing::{error, info, instrument, trace, warn, Instrument, Span};

#[async_trait]
pub trait Job: Send + 'static {
    fn job_type(&self) -> &'static str;
    fn payload(&self) -> Value;
    async fn execute(&self, handle: SchedulerHandle) -> anyhow::Result<()>;
}
enum SchedulerMessage {
    Job(Box<dyn Job>),
    Shutdown(oneshot::Sender<()>),
}
#[derive(Clone, Debug)]
pub struct SchedulerHandle {
    sender: mpsc::Sender<SchedulerMessage>,
    shutdown_rx: watch::Receiver<bool>,
}
impl SchedulerHandle {
    pub async fn submit<J: Job>(&self, job: J) -> Result<(), &'static str> {
        self.sender
            .send(SchedulerMessage::Job(Box::new(job)))
            .await
            .map_err(|_| "Scheduler is shut down")
    }
    pub async fn shutdown(self) {
        let rx = self.internal_shutdown().await;
        rx.await.expect("Scheduler didn't confirm shutdown");
    }
    pub async fn internal_shutdown(&self) -> oneshot::Receiver<()> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(SchedulerMessage::Shutdown(tx))
            .await
            .expect("Scheduler is already gone");
        info!("Scheduler shutdown requested");
        rx
    }
    pub async fn wait_for_shutdown(&mut self) {
        self.shutdown_rx
            .wait_for(|v| *v)
            .await
            .expect("Scheduler dropped without signaling");
    }
}

pub struct Scheduler {
    receiver: mpsc::Receiver<SchedulerMessage>,
    shutdown_tx: watch::Sender<bool>,
}
impl Scheduler {
    pub fn new(buffer: usize) -> (Self, SchedulerHandle) {
        let (sender, receiver) = mpsc::channel(buffer);
        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        let scheduler = Scheduler {
            receiver,
            shutdown_tx,
        };
        let handle = SchedulerHandle {
            sender,
            shutdown_rx,
        };
        (scheduler, handle)
    }
    #[instrument(level = "trace", name = "scheduler", skip_all)]
    pub async fn run(mut self, handle: SchedulerHandle) {
        info!("Scheduler started");
        let mut clean_lever = false;
        while let Some(message) = self.receiver.recv().await {
            match message {
                SchedulerMessage::Job(job) => {
                    if !clean_lever {
                        let job_name = job.job_type();
                        let span = tracing::info_span!("worker", job_name = job_name);
                        span.follows_from(tracing::Span::current());
                        let job_result = job.execute(handle.clone()).instrument(span).await;
                        if job_result.is_err() {
                            let e = job_result.unwrap_err();
                            error!("Failed to execute job: {:?}", e);
                            warn!("A job in the queue has errored out queue will be alive until shutdown hook is called");
                            clean_lever = true;
                        }
                    } else {
                        warn!("Scheduler ignoring job: {}", job.job_type());
                    }
                }
                SchedulerMessage::Shutdown(ack) => {
                    info!("Shutdown hook triggered, draining queue...");
                    self.receiver.close();
                    while let Ok(msg) = self.receiver.try_recv() {
                        if let SchedulerMessage::Job(job) = msg {
                            let job_name = job.job_type();
                            let span = tracing::info_span!("worker-cleanup", job_name = job_name);
                            span.follows_from(tracing::Span::current());
                            let job_result = job.execute(handle.clone()).instrument(span).await;
                            if job_result.is_err() {
                                let e = job_result.unwrap_err();
                                error!("Failed to execute job: {:?}", e);
                            }
                        }
                    }
                    let _ = ack.send(());
                    info!("Scheduler shutdown ack sent");
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
    use crate::job_execution::job_base::{Job, Scheduler, SchedulerHandle};
    use async_trait::async_trait;
    use serde::{Deserialize, Serialize};
    use serde_json::Value;
    use tracing::{info, instrument};

    #[derive(Serialize, Deserialize, Debug)]
    struct PrintJob {
        id: u32,
    }
    #[async_trait]
    impl Job for PrintJob {
        fn job_type(&self) -> &'static str {
            "print-job"
        }
        fn payload(&self) -> Value {
            serde_json::to_value(self).unwrap()
        }
        #[instrument]
        async fn execute(&self, _: SchedulerHandle) -> anyhow::Result<()> {
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
