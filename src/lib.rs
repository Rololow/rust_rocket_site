use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

pub struct ThreadPool {
    thread_wks: Vec<thread_wk>,
    sender: mpsc::Sender<Message>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut thread_wks = Vec::with_capacity(size);

        for id in 0..size {
            thread_wks.push(thread_wk::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { thread_wks, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all thread_wks.");

        for _ in &self.thread_wks {
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down all thread_wks.");

        for thread_wk in &mut self.thread_wks {
            println!("Shutting down thread_wk {}", thread_wk.id);

            if let Some(thread) = thread_wk.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct thread_wk {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl thread_wk {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> thread_wk {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::NewJob(job) => {
                    println!("thread_wk {} got a job; executing.", id);

                    job();
                }
                Message::Terminate => {
                    println!("thread_wk {} was told to terminate.", id);

                    break;
                }
            }
        });

        thread_wk {
            id,
            thread: Some(thread),
        }
    }
}
