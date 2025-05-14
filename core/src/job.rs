use serde::{Deserialize, Serialize};
use std::fs;
use log::{debug, info, error};
use bollard::Docker;
use bollard::container::{Config, CreateContainerOptions, StartContainerOptions, LogsOptions, LogOutput};
use bollard::models::HostConfig;
use futures::StreamExt;
use std::fs::File;
use std::io::Write;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub name: Option<String>,
    pub image: String,
    pub command: Vec<String>,
    pub environment: Option<Vec<String>>,
    pub artifacts: Option<String>,
    pub schedule: Option<String>,
    pub triggers: Option<Vec<String>>,
    pub dependencies: Option<Vec<String>>
}


impl Job {
    pub async fn run(&self, docker: &Docker) {
        info!("starting job {}", self.name.clone().unwrap());

        // // Pull the image if not present
        // if let Err(e) = self.pull_image(docker).await {
        //     error!("Error pulling image: {}", e);
        //     return;
        // }

        // // Create container
        
        // let mounts = match self.prepare_mounts(&self.name.clone().unwrap()) {
        //     Ok(m) => m,
        //     Err(e) => {
        //         error!("Failed to prepare mounts: {}", e);
        //         return;
        //     }
        // };


        // let container = match self.create_container(docker, &self.name.clone().unwrap(), mounts).await {
        //     Ok(id) => id,
        //     Err(e) => {
        //         error!("Error creating container: {}", e);
        //         return;
        //     }
        // };

        // // Start the container
        // info!("Starting container: {}", container);
        // if let Err(e) = docker.start_container(&container, None::<StartContainerOptions<String>>).await {
        //     error!("Error starting container: {}", e);
        //     return;
        // }

        // // Get logs
        // if let Err(e) = self.stream_logs(docker, &container, &self.name.clone().unwrap()).await {
        //     error!("Failed to stream logs: {}", e);
        //     return;
        // }
        info!("Job {} completed.", &self.name.clone().unwrap());
    }
    
    async fn pull_image(&self, docker: &Docker) -> Result<(), bollard::errors::Error> {
        debug!("downloading image {}", self.image);

        let mut stream = docker.create_image(
            Some(bollard::image::CreateImageOptions {
                from_image: self.image.clone(),
                ..Default::default()
            }),
            None,
            None
        );

        while let Some(result) = stream.next().await {
            if let Ok(output) = result {
                if let Some(status) = output.status {
                    info!("Pull status: {}", status);
                }
            } else {
                return Err(result.err().unwrap());
            }
        }

        Ok(())
    }

    fn prepare_mounts(&self, job_name: &str) -> Result<Vec<String>, std::io::Error> {
        let mut mounts: Vec<String> = Vec::new();

        if let Some(artifacts_path) = &self.artifacts {
            let mount_path = format!("/tmp/oxipipe/artifacts/{}/workspace", job_name);
            fs::create_dir_all(&mount_path)?;
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

        Ok(mounts)
    }

    async fn create_container(
        &self,
        docker: &Docker,
        name: &str,
        mounts: Vec<String>,
    ) -> Result<String, bollard::errors::Error> {
        let container_name = format!("oxipipe-{}", name);

        let config = Config {
            image: Some(self.image.clone()),
            cmd: Some(self.command.clone()),
            env: self.environment.clone(),
            host_config: Some(HostConfig {
                auto_remove: Some(true),
                binds: Some(mounts),
                ..Default::default()
            }),
            ..Default::default()
        };

        let create_container_options = Some(CreateContainerOptions { 
            name: &container_name,
            platform: None,
        });

        let result = docker.create_container(create_container_options, config).await?;

        Ok(result.id)
    }

    async fn stream_logs(
        &self,
        docker: &Docker,
        container_id: &str,
        job_name: &str
    ) -> Result<(), bollard::errors::Error> {

        let mut logs = docker.logs(
            &container_id,
            Some(LogsOptions::<String> {
                follow: true,
                stdout: true,
                stderr: true,
                ..Default::default()
            }),
        );

        let log_dir = format!("/tmp/oxipipe/artifacts/{}/logs", job_name);
        fs::create_dir_all(&log_dir)?;

        let log_file = format!("{}/output.log", log_dir);
        let mut file = File::create(&log_file)?;

        let mut wrote_output = false;

        while let Some(log_line) = logs.next().await {
            if let Ok(line) = log_line {
                debug!("{}", line);
                write!(file, "{}", line.to_string())?;
                file.flush()?;
                wrote_output = true;
            }
        }

        if !wrote_output {
            writeln!(file, "No output")?;
            file.flush()?;
        }
        
        Ok(())
    }

}
