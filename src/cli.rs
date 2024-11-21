use clap::{Parser, arg};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "pdh", version = "0.1.0", author = "Lukas Sanner")]
#[command(about = "A tool to detect and classify faces in images")]
pub struct Args {
    #[arg(required = true, help = "The path(s) to the directory containing the images")]
    pub directories: Vec<PathBuf>,
}