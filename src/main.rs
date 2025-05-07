use log::{info, error};
use std::collections::VecDeque;
use bollard::Docker;

mod pipeline;
mod job;

use pipeline::Pipeline;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    env_logger::init();
    info!("Starting OxiPipe...");

    let docker = Docker::connect_with_local_defaults().unwrap();

    let mut queue = VecDeque::new();

    let pipeline = Pipeline::from_file("examples/hello.yml")?;

    let start_job = "hello".to_string();
    
    let job = pipeline.jobs.get(&start_job).unwrap();
    queue.push_back((start_job.clone(), job));

    while let Some((name, job)) = queue.pop_front() {
        job.run(&docker, name).await;
        if let Some(triggers) = &job.triggers {
            for triggered_job_name in triggers {
                if let Some(triggered_job) = pipeline.jobs.get(triggered_job_name) {
                    queue.push_back((triggered_job_name.clone(), triggered_job));
                } else {
                    error!("Triggered job '{}' not found.", triggered_job_name);
                }
            }
        }
    }

    Ok(())
}
