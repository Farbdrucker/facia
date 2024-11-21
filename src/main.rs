mod cli;
mod schema;
mod tasks;
mod facenet;

use std::time::Instant;
use clap::Parser;

use tch::{Device, nn};

use crate::tasks::collect_files;
use crate::tasks::detect;
use crate::cli::Args;
use crate::facenet::inception_resnetv1::InceptionResnetV1;

fn main() {
    env_logger::init();

    // parse the arguments
    let args = Args::parse();
    let dirs = args.directories;

    // init pytorch facenet model
    let vs = nn::VarStore::new(Device::cuda_if_available());
    let model = InceptionResnetV1::new(&vs.root());

    // start timer to track elapsed time
    let start_time = Instant::now();

    // collect all the image files (most computation done here)
    let files = collect_files::run(&dirs).unwrap();

    // TODO add face detection
    let detection_results = detect::run(&files).unwrap();

    // TODO add face embedding

    // TODO add face classification

    // TODO add DB syncing


    // compute elapsed time
    let elapsed_time = start_time.elapsed();
    println!("Total execution time: {:?} for {} files", elapsed_time, files.len());
    let faces = detection_results.iter().map(|detection_result| detection_result.detections.len()).sum::<usize>();
    println!("Number of faces found: {:?}",faces );

    // compute avg time per file 
    if !files.is_empty() {
        let time_per_file = elapsed_time / detection_results.len() as u32;
        println!("Time per file: {:?}", time_per_file);
    } else {
        println!("No files processed.");
    }
}
