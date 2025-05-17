use clap::Parser;
use tracing::{info, error};
use std::collections::{HashMap};
use dotenv::dotenv;

mod schedule;

mod args;
mod chroniq;
mod web;
mod scheduler;

use args::Args;
use core::pipeline::Pipeline;
use schedule::Schedule;
use core::job_queue::JobQueue;

use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    info!("Starting OxiPipe...");

    dotenv().ok();
    
    let args = Args::parse();

    let mut pipeline = Pipeline::from_file(&args.pipeline)?;

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

    for job in args.jobs {
        if let Some(triggered_job) = pipeline.jobs.get(&job) {
            queue.enqueue(triggered_job.clone()).await;
            info!("{} triggered", job);
        } else {
            error!("Triggered job '{}' not found.", job);
        }
    }
    
    tokio::join!(
        chroniq::start(schedules, queue.clone(), pipeline.clone()),
        web::start(),
        scheduler::start(queue.clone(), pipeline, !args.no_trigger)
    );

    Ok(())
}
