use std::sync::{mpsc::{Receiver, Sender}, Mutex};
use serde_json::Value;

/// The job trait should be implemented for all jobs that
/// should be runnable in the job queue.
pub trait Job {
    fn execute(&mut self);
    fn get_result(&self) -> Value;
}
#[derive(Debug)]
pub struct JobQueue {
    receiver: Mutex<Option<Receiver<Box<dyn Job>>>>,
    sender: Mutex<Option<Sender<Box<dyn Job>>>>
}
#[derive(Clone)]
pub struct ResultWrapper {
    result: Value,
}
impl ResultWrapper {
    pub fn create_result(result: Value) -> ResultWrapper {
        ResultWrapper { result }
    }
    pub fn get_result(&self) -> Value {
        self.result.clone()
    }
}
impl JobQueue {
    pub fn new() -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        JobQueue {
            receiver: Mutex::new(Some(receiver)),
            sender: Mutex::new(Some(sender))
        }
    }
    pub fn write(&self, job: Box<dyn Job>) {
        let sender = self.sender.lock().unwrap();
        sender.as_ref().unwrap().send(job).unwrap();
    }
    pub fn read(&self) -> Box<dyn Job> {
        let mut receiver = self.receiver.lock().unwrap();
        receiver.as_mut().unwrap().recv().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};
    use serde::{Deserialize, Serialize};
    use super::*;

    struct AddJob {
        a: i32,
        b: i32,
        result: Option<ResultWrapper>
    }
    impl AddJob {
        fn new_add_job(a: i32, b: i32) -> AddJob {
            AddJob {
                a,
                b,
                result: None
            }
        }
    }
    #[derive(Deserialize, Serialize)]
    struct Result {
        addition_result: i32
    }
    impl Job for AddJob {
        fn execute(&mut self) {
            self.result = Some(ResultWrapper::create_result(json!({
                "addition_result": self.a + self.b
            })));
            println!("{} + {} resulted in a response of: {}", self.a, self.b, self.result.clone().unwrap().get_result());
        }
        fn get_result(&self) -> Value {
            self.result.clone().unwrap().get_result()
        }
    }

    #[test]
    fn test_queue_and_de_queue() {
        let job_queue = JobQueue::new();
        job_queue.write(Box::new(AddJob::new_add_job(2, 2)));
        let mut job = job_queue.read();
        job.execute();
        let add_job_res: Result = serde_json::from_value(job.get_result()).unwrap();
        assert_eq!(add_job_res.addition_result, 4);
    }
}
