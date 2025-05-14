
use core::{job_queue::JobQueue};
use bollard::{Docker};
use tracing::{info, error};

use core::pipeline::Pipeline;

pub async fn start(queue: JobQueue, pipeline:Pipeline) {
    let docker = Docker::connect_with_local_defaults().unwrap();
    loop {
        queue.wait_for_job().await;
        if let Some(job) = queue.next_job().await {
            info!("Starting job: {}", job.clone().name.unwrap());
            // run_job(&docker, job, name).await;
            job.run(&docker).await;
            if let Some(triggers) = &job.triggers {
                for triggered_job_name in triggers {
                    if let Some(triggered_job) = pipeline.jobs.get(triggered_job_name) {
                        queue.enqueue(triggered_job.clone()).await;
                        info!("{} triggered", triggered_job_name);
                    } else {
                        error!("Triggered job '{}' not found.", triggered_job_name);
                    }
                }
            }
            queue.job_done().await;
        }
    }

}