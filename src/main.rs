use log::{info, error};
use bollard::Docker;
use bollard::container::{Config, CreateContainerOptions, StartContainerOptions, LogsOptions};
use bollard::models::HostConfig;
use futures::StreamExt;
use std::collections::{HashMap, VecDeque};
use std::path::Path;
use std::fs;

mod pipeline;
mod job;
mod schedule;
mod job_queue;

mod chroniq;
mod web;
mod scheduler;

use pipeline::Pipeline;
use job::Job;
use schedule::Schedule;
use job_queue::JobQueue;

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
