use log::{debug, info, error};
use serde::{Deserialize, Serialize};
use bollard::Docker;
use bollard::container::{Config, CreateContainerOptions, StartContainerOptions, LogsOptions};
use bollard::models::HostConfig;
use futures::StreamExt;

#[derive(Debug, Serialize, Deserialize)]
struct Job {
    id: String,
    image: String,
    command: Vec<String>,
    env: Option<Vec<(String, String)>>
}

async fn run_job(docker: &Docker, job: &Job) {
    info!("starting job {}", job.id);

    // Pull the image if not present
    debug!("downloading image {}", job.image);
    let mut pull_stream = docker.create_image(
        Some(bollard::image::CreateImageOptions {
            from_image: job.image.clone(),
            ..Default::default()
        }),
        None,
        None
    );

    while let Some(pull_result) = pull_stream.next().await {
        match pull_result {
            Ok(output) => {
                if let Some(status) = output.status {
                    info!("Pull status: {}", status);
                }
            },
            Err(e) => {
                error!("Error pulling image: {}", e);
                return;
            }
        }
    }

    // Prepare environment
    let env_vars = job.env.as_ref().map(|pairs| {
        pairs.iter().map(|(k, v)| format!("{}={}", k, v)).collect()
    });
    
    // Create container
    let container_name = format!("oxipipe-{}", job.id);
    let create_result = docker
        .create_container(
            Some(CreateContainerOptions { 
                name: &container_name,
                platform: None,
            }),
            Config {
                image: Some(job.image.clone()),
                cmd: Some(job.command.clone()),
                env: env_vars,
                host_config: Some(HostConfig {
                    auto_remove: Some(true), // automatically clean up
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .await;

    let container = match create_result {
        Ok(info) => info.id,
        Err(e) => {
            error!("Error creating container: {}", e);
            return;
        }
    };

    // Start the container
    info!("Starting container: {}", container);
    if let Err(e) = docker.start_container(&container, None::<StartContainerOptions<String>>).await {
        error!("Error starting container: {}", e);
        return;
    }

    // Get logs
    let mut logs = docker.logs(
        &container,
        Some(LogsOptions::<String> {
            follow: true,
            stdout: true,
            stderr: true,
            ..Default::default()
        }),
    );

    debug!("Logs:");
    while let Some(log_line) = logs.next().await {
        if let Ok(line) = log_line {
            info!("{}", line);
        }
    }

    info!("Job {} completed.", job.id);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    env_logger::init();
    info!("Starting OxiPipe...");

    let docker = Docker::connect_with_local_defaults().unwrap();

    let job = Job {
        id: "example-job-1".to_string(),
        image: "alpine:latest".to_string(),
        command: vec!["echo".to_string(), "Hello, from OxiPipe!".to_string()],
        env: None,
    };

    run_job(&docker, &job).await;
    

    Ok(())
}
