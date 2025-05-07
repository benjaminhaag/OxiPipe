use log::{info, error};
use bollard::Docker;
use bollard::container::{Config, CreateContainerOptions, StartContainerOptions, LogsOptions};
use bollard::models::HostConfig;
use futures::StreamExt;
use std::collections::{HashMap, VecDeque};
use std::path::Path;
use std::fs;
use tokio::time::{sleep, Duration};

mod pipeline;
mod job;
mod schedule;

use pipeline::Pipeline;
use job::Job;
use schedule::Schedule;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    env_logger::init();
    info!("Starting OxiPipe...");

    let docker = Docker::connect_with_local_defaults().unwrap();

    let mut queue: VecDeque<Job> = VecDeque::new();

    let pipeline = Pipeline::from_file("examples/cron.yml")?;

    let mut schedules: HashMap<String, Schedule> = HashMap::new();
    for (name, job) in &pipeline.jobs {
        if let Some(raw_schedule) = &job.schedule {
            let parsed = Schedule::from_str(raw_schedule)?;
            schedules.insert(name.clone(), parsed);
        }
    }

    //let start_job = "hello".to_string();
    
    //let job = pipeline.jobs.get(&start_job).unwrap();
    //queue.push_back((start_job.clone(), job));

    //while let Some((name, job)) = queue.pop_front() {
    //    run_job(&docker, job, name).await;
    //    if let Some(triggers) = &job.triggers {
    //        for triggered_job_name in triggers {
    //            if let Some(triggered_job) = pipeline.jobs.get(triggered_job_name) {
    //                queue.push_back((triggered_job_name.clone(), triggered_job));
    //            } else {
    //                error!("Triggered job '{}' not found.", triggered_job_name);
    //            }
    //        }
    //    }
    //}
    
    loop {
        for (name, schedule) in &mut schedules {
            if schedule.should_run() {
                println!("{} triggered", name);
            }
        }
        sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}
