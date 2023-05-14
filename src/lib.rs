#![allow(dead_code)]
#![allow(unused_variables)]

use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send>;

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(worker_id: usize, reciever: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            match reciever.lock().unwrap().recv() {
                Ok(job) => {
                    println!("Worker with id: {} received job", worker_id);
                    job();
                }
                Err(_) => {
                    println!("Shutting down loop of worker id: {worker_id}");
                    break;
                }
            }
        });

        println!("Worker with id: {worker_id} constructed");

        Worker {
            id: worker_id,
            thread: Some(thread),
        }
    }
}

impl ThreadPool {
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }

    pub fn new(threads: usize) -> ThreadPool {
        let (sender, reciever) = mpsc::channel();
        let reciever = Arc::new(Mutex::new(reciever));

        let mut workers = Vec::with_capacity(threads);

        for thread_id in 0..threads {
            println!("thread_id: {}", thread_id);
            workers.push(Worker::new(thread_id, reciever.clone()));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            println!("Shutting down worker: {}", worker.id);
            worker.thread.take().unwrap().join().unwrap();
        }
    }
}
