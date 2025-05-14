use tokio::time::{sleep, Duration};
use tracing::info;
use std::collections::{HashMap};


use core::pipeline::Pipeline;
use crate::schedule::Schedule;
use core::job_queue::JobQueue;

pub async fn start(mut schedules: HashMap<String, Schedule>, queue: JobQueue, pipeline: Pipeline) {
    loop {
        for (name, schedule) in &mut schedules {
            if schedule.should_run() {
                let job = pipeline.jobs.get(name).unwrap();
                queue.enqueue(job.clone()).await;
                info!("{} triggered", name);
                
            }
        }
        sleep(Duration::from_secs(1)).await;
    }
}