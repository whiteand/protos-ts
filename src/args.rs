use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct CliArguments {
    /// Proto files directory path
    pub src: PathBuf,

    /// Output directory into which the files
    /// will be generated
    #[arg(short, long, value_name = "DIR_PATH")]
    pub out: PathBuf,
}
