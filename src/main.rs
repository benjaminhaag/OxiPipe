use log::{debug, info, error};
use serde::{Deserialize, Serialize};
use serde_yaml;
use bollard::Docker;
use bollard::container::{Config, CreateContainerOptions, StartContainerOptions, LogsOptions};
use bollard::models::HostConfig;
use futures::StreamExt;
use std::collections::{HashMap, VecDeque};
use std::path::Path;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
struct Job {
    image: String,
    command: Vec<String>,
    environment: Option<Vec<String>>,
    artifacts: Option<String>,
    triggers: Option<Vec<String>>,
    dependencies: Option<Vec<String>>
}

#[derive(Debug, Serialize, Deserialize)]
struct Pipeline {
    jobs: HashMap<String, Job>
}

async fn run_job(docker: &Docker, job: &Job, name: String) {
    info!("starting job {}", name);

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

    // Create container
    let container_name = format!("oxipipe-{}", name);
    
    let mut mounts: Vec<String> = Vec::new();

    if let Some(artifacts_path) = &job.artifacts {
        let mount_path = format!("/tmp/oxipipe/artifacts/{}", name);
        fs::create_dir_all(&mount_path).unwrap();
        let mount = format!("{}:{}",mount_path, artifacts_path);
        mounts.push(mount);
    }

    if let Some(dependencies) = &job.dependencies {
        for dep in dependencies {
            let dep_artifact_path = format!("/tmp/oxipipe/artifacts/{}", dep);
            let mount = format!("{}:/artifacts/{}", dep_artifact_path, dep);
            mounts.push(mount);
        }
    }

    let create_result = docker
        .create_container(
            Some(CreateContainerOptions { 
                name: &container_name,
                platform: None,
            }),
            Config {
                image: Some(job.image.clone()),
                cmd: Some(job.command.clone()),
                env: job.environment.clone(),
                host_config: Some(HostConfig {
                    auto_remove: Some(true),
                    binds: Some(mounts),
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
    if let Err(e) = docker.start_container(&container, None::<StartContainerOptions<String>>).await {  error!("Error starting container: {}", e);
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

    info!("Job {} completed.", name);
}

fn load_pipeline_from_file(file_path: &str) -> Result<Pipeline, Box<dyn std::error::Error>> {
    let yaml_str = std::fs::read_to_string(file_path)?;
    let pipeline: Pipeline = serde_yaml::from_str(&yaml_str)?;
    Ok(pipeline)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    env_logger::init();
    info!("Starting OxiPipe...");

    let docker = Docker::connect_with_local_defaults().unwrap();

    let mut queue = VecDeque::new();

    let pipeline = load_pipeline_from_file("examples/hello.yml")?;

    let start_job = "hello".to_string();
    
    let job = pipeline.jobs.get(&start_job).unwrap();
    queue.push_back((start_job.clone(), job));

    while let Some((name, job)) = queue.pop_front() {
        run_job(&docker, job, name).await;
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
