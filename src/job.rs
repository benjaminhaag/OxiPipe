use serde::{Deserialize, Serialize};
use std::fs;
use log::{debug, info, error};
use bollard::Docker;
use bollard::container::{Config, CreateContainerOptions, StartContainerOptions, LogsOptions};
use bollard::models::HostConfig;
use futures::StreamExt;

#[derive(Debug, Serialize, Deserialize)]
pub struct Job {
    pub image: String,
    pub command: Vec<String>,
    pub environment: Option<Vec<String>>,
    pub artifacts: Option<String>,
    pub triggers: Option<Vec<String>>,
    pub dependencies: Option<Vec<String>>
}


impl Job {
    pub async fn run(&self, docker: &Docker, name: String) {
        info!("starting job {}", name);

        // Pull the image if not present
        debug!("downloading image {}", self.image);
        let mut pull_stream = docker.create_image(
            Some(bollard::image::CreateImageOptions {
                from_image: self.image.clone(),
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

        if let Some(artifacts_path) = &self.artifacts {
            let mount_path = format!("/tmp/oxipipe/artifacts/{}/workspace", name);
            fs::create_dir_all(&mount_path).unwrap();
            let mount = format!("{}:{}",mount_path, artifacts_path);
            mounts.push(mount);
        }

        if let Some(dependencies) = &self.dependencies {
            for dep in dependencies {
                let dep_artifact_path = format!("/tmp/oxipipe/artifacts/{}/workspace", dep);
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
                    image: Some(self.image.clone()),
                    cmd: Some(self.command.clone()),
                    env: self.environment.clone(),
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

}
