use std::sync::{mpsc::{Receiver, Sender}, Mutex};

pub trait Job {
    fn execute(&mut self);
    fn get_result(&self) -> String;
}

#[derive(Debug)]
pub struct JobQueue {
    receiver: Mutex<Option<Receiver<Box<dyn Job>>>>,
    sender: Mutex<Option<Sender<Box<dyn Job>>>>
}

impl JobQueue {
    pub fn new() -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        JobQueue {
            receiver: Mutex::new(Some(receiver)),
            sender: Mutex::new(Some(sender))
        }
    }

    pub fn push(&self, job: Box<dyn Job>) {
        let sender = self.sender.lock().unwrap();
        sender.as_ref().unwrap().send(job).unwrap();
    }

    pub fn pop(&self) -> Box<dyn Job> {
        let mut receiver = self.receiver.lock().unwrap();
        receiver.as_mut().unwrap().recv().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct AddJob {
        a: i32,
        b: i32,
        result: i32
    }

    impl Job for AddJob {
        fn execute(&mut self) {
            self.result = self.a + self.b;
            println!("{} + {} = {}", self.a, self.b, self.result);
        }
        fn get_result(&self) -> String {
            String::from(format!("{}", self.result))
        }
    }

    #[test]
    fn test_queue_and_de_queue() {
        let job_queue = JobQueue::new();
        let job = Box::new(AddJob { a: 2, b: 2, result: 0 });
        job_queue.push(job);
        let mut job = job_queue.pop();
        job.execute();
        assert_eq!(job.get_result(), String::from("4"));
    }
}
