use log::{info};
use std::collections::{HashMap};

mod schedule;

mod chroniq;
mod web;
mod scheduler;

use core::pipeline::Pipeline;
use schedule::Schedule;
use core::job_queue::JobQueue;

use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    tracing_subscriber::fmt::init();

    info!("Starting OxiPipe...");

    let mut pipeline = Pipeline::from_file("examples/cron.yml")?;

    let mut schedules: HashMap<String, Schedule> = HashMap::new();
    for (name, job) in &mut pipeline.jobs {
        if job.name == None {
            job.name = Some(name.clone());
        }
        if let Some(raw_schedule) = &job.schedule {
            let parsed = Schedule::from_str(raw_schedule)?;
            schedules.insert(name.clone(), parsed);
        }
    }

    let queue = JobQueue::new(1);
    
    tokio::join!(
        chroniq::start(schedules, queue.clone(), pipeline.clone()),
        web::start(),
        scheduler::start(queue.clone(), pipeline)
    );

    Ok(())
}
