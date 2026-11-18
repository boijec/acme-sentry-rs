use crate::error::JobEngineError;
use crate::job::{Job, JobContext, JobExecutionFlag};
use crate::scheduling::Scheduler;
use async_trait::async_trait;
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;
use tracing::instrument;

#[derive(Clone)]
struct Deps;

fn test_deps() -> Deps {
    Deps {}
}

type Log = mpsc::UnboundedSender<String>;

fn drain(rx: &mut mpsc::UnboundedReceiver<String>) -> Vec<String> {
    let mut out = Vec::new();
    while let Ok(m) = rx.try_recv() {
        out.push(m);
    }
    out
}

struct PrintJob {
    id: u32,
    log: Log,
}

#[async_trait]
impl<Deps: Clone + Send + Sync + 'static> Job<Deps> for PrintJob {
    fn job_type(&self) -> &str {
        "print-job"
    }

    #[instrument(skip(self, _ctx))]
    async fn execute(
        self: Box<Self>,
        _ctx: JobContext<Deps>,
    ) -> Result<JobExecutionFlag, JobEngineError> {
        let _ = self.log.send(format!("start {}", self.id));
        tokio::time::sleep(Duration::from_millis(500)).await;
        let _ = self.log.send(format!("end {}", self.id));
        Ok(JobExecutionFlag::Next)
    }
}

struct StopJob {
    log: Log,
}

#[async_trait]
impl<Deps: Clone + Send + Sync + 'static> Job<Deps> for StopJob {
    fn job_type(&self) -> &str {
        "stop-job"
    }

    async fn execute(
        self: Box<Self>,
        _ctx: JobContext<Deps>,
    ) -> Result<JobExecutionFlag, JobEngineError> {
        let _ = self.log.send("stop".into());
        Ok(JobExecutionFlag::Stop)
    }
}

struct ChainJob {
    log: Log,
}

#[async_trait]
impl<Deps: Clone + Send + Sync + 'static> Job<Deps> for ChainJob {
    fn job_type(&self) -> &str {
        "chain-job"
    }

    async fn execute(
        self: Box<Self>,
        ctx: JobContext<Deps>,
    ) -> Result<JobExecutionFlag, JobEngineError> {
        let _ = self.log.send("parent".into());
        ctx.handle
            .submit(
                StopJob {
                    log: self.log.clone(),
                },
                None,
            )
            .await?;
        Ok(JobExecutionFlag::Next)
    }
}

struct WaitForCancelJob {
    log: Log,
}

#[async_trait]
impl<Deps: Clone + Send + Sync + 'static> Job<Deps> for WaitForCancelJob {
    fn job_type(&self) -> &str {
        "wait-for-cancel"
    }

    async fn execute(
        self: Box<Self>,
        ctx: JobContext<Deps>,
    ) -> Result<JobExecutionFlag, JobEngineError> {
        let _ = self.log.send("running".into());
        ctx.token.cancelled().await;
        let _ = self.log.send("cancelled".into());
        Ok(JobExecutionFlag::Next)
    }
}

#[tokio::test(start_paused = true)]
async fn jobs_run_serially_then_complete() {
    let (log, mut log_rx) = mpsc::unbounded_channel();
    let (mut scheduler, handle) = Scheduler::new("some-user-id", 32);
    scheduler.with_dependencies(test_deps());
    tokio::spawn(scheduler.run());

    handle
        .submit(
            PrintJob {
                id: 1,
                log: log.clone(),
            },
            None,
        )
        .await
        .unwrap();
    handle
        .submit(
            StopJob { log: log.clone() },
            Some(SystemTime::now() + Duration::from_secs(2)),
        )
        .await
        .unwrap();

    handle.wait_for_completion().await.unwrap();

    assert_eq!(drain(&mut log_rx), ["start 1", "end 1", "stop"]);
}

#[tokio::test(start_paused = true)]
async fn delayed_job_runs_later_without_blocking() {
    let (log, mut log_rx) = mpsc::unbounded_channel();
    let (mut scheduler, handle) = Scheduler::new("some-user-id", 32);
    scheduler.with_dependencies(test_deps());
    tokio::spawn(scheduler.run());

    handle
        .submit(
            PrintJob {
                id: 1,
                log: log.clone(),
            },
            Some(SystemTime::now() + Duration::from_secs(10)),
        )
        .await
        .unwrap();
    handle
        .submit(
            PrintJob {
                id: 2,
                log: log.clone(),
            },
            None,
        )
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_secs(5)).await;
    assert_eq!(
        drain(&mut log_rx),
        ["start 2", "end 2"],
        "job 1 must not run early"
    );

    tokio::time::sleep(Duration::from_secs(10)).await;
    assert_eq!(drain(&mut log_rx), ["start 1", "end 1"]);

    handle.shutdown().await;
}

#[tokio::test(start_paused = true)]
async fn job_can_submit_followup_via_job_handle() {
    let (log, mut log_rx) = mpsc::unbounded_channel();
    let (mut scheduler, handle) = Scheduler::new("some-user-id", 32);
    scheduler.with_dependencies(test_deps());
    tokio::spawn(scheduler.run());

    handle
        .submit(ChainJob { log: log.clone() }, None)
        .await
        .unwrap();
    handle.wait_for_completion().await.unwrap();

    assert_eq!(drain(&mut log_rx), ["parent", "stop"]);
}

#[tokio::test(start_paused = true)]
async fn shutdown_cancels_running_jobs_and_waits() {
    let (log, mut log_rx) = mpsc::unbounded_channel();
    let (mut scheduler, handle) = Scheduler::new("some-user-id", 32);
    scheduler.with_dependencies(test_deps());
    tokio::spawn(scheduler.run());

    handle
        .submit(WaitForCancelJob { log: log.clone() }, None)
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(10)).await;
    assert_eq!(drain(&mut log_rx), ["running"]);

    handle.clone().shutdown().await;

    assert_eq!(drain(&mut log_rx), ["cancelled"]);
    handle.wait_for_shutdown().await;
    assert!(handle.submit(StopJob { log }, None).await.is_err());
}
