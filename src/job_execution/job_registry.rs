use crate::job_execution::job_base::Job;
use std::collections::HashMap;

#[allow(unused)]
type JobFactory = Box<dyn Fn(serde_json::Value) -> Box<dyn Job> + Send + Sync>;

#[allow(unused)]
pub struct JobRegistry {
    factories: HashMap<&'static str, JobFactory>,
}

#[allow(unused)]
impl JobRegistry {
    pub fn new() -> Self {
        Self { factories: HashMap::new() }
    }
    pub fn register<J, F>(&mut self, job_type: &'static str, factory: F)
    where
        J: Job,
        F: Fn(serde_json::Value) -> J + Send + Sync + 'static,
    {
        self.factories.insert(
            job_type,
            Box::new(move |payload| Box::new(factory(payload))),
        );
    }
    pub fn reconstruct(&self, job_type: &str, payload: serde_json::Value) -> Option<Box<dyn Job>> {
        self.factories.get(job_type).map(|f| f(payload))
    }
}