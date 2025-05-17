use crate::job::Job;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};
// use uuid::Uuid;

#[derive(Clone)]
pub struct JobQueue {
    inner: Arc<Mutex<Inner>>,
    notify: Arc<Notify>,
}

struct Inner {
    queue: VecDeque<Job>,
    running: usize,
    max_concurrent: usize,
    // scheduled_ids: HashMap<String, Uuid>, // for dedup/debounce
}

impl JobQueue {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                queue: VecDeque::new(),
                running: 0,
                max_concurrent,
                // scheduled_ids: HashMap::new(),
            })),
            notify: Arc::new(Notify::new()),
        }
    }

    pub async fn enqueue(&self, job: Job) {
        let mut inner = self.inner.lock().await;

        // Debounce: only one job with the same name
        // if inner.scheduled_ids.contains_key(&job.name) {
        //     return;
        // }

        // let id = Uuid::new_v4();
        //inner.scheduled_ids.insert(job.name.clone(), id);
        inner.queue.push_back(job);
        self.notify.notify_one();
    }

    // pub async fn unschedule(&self, job_name: &str) {
    //     let mut inner = self.inner.lock().await;
    //     inner.queue.retain(|job| job.name != job_name);
    //     inner.scheduled_ids.remove(job_name);
    // }

    pub async fn next_job(&self) -> Option<Job> {
        let mut inner = self.inner.lock().await;

        if inner.running >= inner.max_concurrent {
            return None;
        }

        if let Some(job) = inner.queue.pop_front() {
            inner.running += 1;
            // inner.scheduled_ids.remove(&job.name);
            Some(job)
        } else {
            None
        }
    }

    pub async fn job_done(&self) {
        let mut inner = self.inner.lock().await;
        if inner.running > 0 {
            inner.running -= 1;
        }
        self.notify.notify_one();
    }

    pub async fn wait_for_job(&self) {
        self.notify.notified().await;
    }
}