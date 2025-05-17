use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {

    /// Path to the pipeline YAML file
    #[arg(short = 'p', long = "pipeline")]
    pub pipeline: String,

    /// List of jobs to schedule (optional)
    #[arg(short = 'j', long = "jobs")]
    pub jobs: Vec<String>,

    // Disable job triggering (optional)
    #[arg(long = "no-trigger", default_value_t = false)]
    pub no_trigger: bool,
}