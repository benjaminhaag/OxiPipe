use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::job::Job;


#[derive(Debug, Serialize, Deserialize)]
pub struct Pipeline {
    pub jobs: HashMap<String, Job>
}

impl Pipeline {

    pub fn from_file(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let yaml_str = std::fs::read_to_string(file_path)?;
        let pipeline: Pipeline = serde_yaml::from_str(&yaml_str)?;
        Ok(pipeline)
    }

}
