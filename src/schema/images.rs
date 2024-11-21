// schema/images.rs
use chrono::{DateTime, Utc};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Image {
    pub path: PathBuf,
    pub creation_timestamp: DateTime<Utc>,
    pub processing_timestamp: DateTime<Utc>,
    pub hash: String,
}