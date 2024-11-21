// tasks/collect_files.rs

use std::{fs::ReadDir, path::{Path, PathBuf}};
use chrono::{DateTime, Utc};
use log::{debug, info};
use rayon::prelude::*;
use std::fs;
use sha2::{Sha256, Digest};
use walkdir::WalkDir;

use crate::schema::images::Image;

fn is_valid_image_file(path: &Path) -> bool {
    let valid_extensions = ["jpg", "jpeg", "JPG", "JPEG", "heic", "HEIC"];
    path.extension()
        .and_then(|ext| ext.to_str())
        .map_or(false, |ext| valid_extensions.contains(&ext))
}

fn get_creation_timestamp(path: &Path) -> Option<DateTime<Utc>> {
    path.metadata().ok().and_then(|metadata| {
        metadata.created().ok().map(|time| time.into())
    })
}

fn compute_sha256_hash(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let file = fs::File::open(path)?;
    let mut buf_reader = std::io::BufReader::new(file);
    let mut hasher = Sha256::new();
    std::io::copy(&mut buf_reader, &mut hasher)?;
    Ok(format!("{:x}", hasher.finalize()))
}

fn create_hashed_image_file(path: &Path) -> Result<Image, Box<dyn std::error::Error>> {
    let today_utc = Utc::now();
    let timestamp = get_creation_timestamp(path).unwrap_or(today_utc);
    let hash = compute_sha256_hash(path)?;
    Ok(Image {
        path: path.to_path_buf(),
        creation_timestamp: timestamp,
        processing_timestamp: today_utc,
        hash,
    })
}

fn collect_files(dirs: &Vec<PathBuf>) -> Result<Vec<Image>, Box<dyn std::error::Error>> {
    info!("Collecting subdirectories in {} directories...", dirs.len());
    
    let all_dirs: Vec<PathBuf> = dirs.par_iter()
        .flat_map(|dir| {
            WalkDir::new(dir)
                .into_iter()
                .filter_entry(|entry| entry.file_type().is_dir())
                .filter_map(Result::ok)
                .map(|entry| entry.path().to_path_buf())
                .collect::<Vec<PathBuf>>()
        })
        .collect();
    
    info!("Collecting files in {} directories...", all_dirs.len());

    let mut images = Vec::new();

    for dir in &all_dirs {
        match fs::read_dir(dir) {
            Ok(read_dir) => {
                for entry_result in read_dir {
                    if let Ok(entry) = entry_result {
                        let path = entry.path();
                        if is_valid_image_file(&path) {
                            match create_hashed_image_file(&path) {
                                Ok(image) => images.push(image),
                                Err(e) => {
                                    debug!("Failed to process file {}: {}", path.display(), e);
                                }
                            }
                        }
                    } else {
                        debug!("Failed to read directory entry: {}", entry_result.unwrap_err());
                    }
                }
            },
            Err(e) => {
                debug!("Failed to read directory {}: {}", dir.display(), e);
            }
        }
    }

    Ok(images)
}

pub fn run(dirs: &Vec<PathBuf>) -> Result<Vec<Image>, Box<dyn std::error::Error>> {
    collect_files(&dirs)
}