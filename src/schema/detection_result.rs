use chrono::{DateTime, Utc};
use rust_faces::Rect;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct BBox {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl BBox {
    pub fn from_rect(rect: Rect) -> Self {
        BBox {
            x: rect.x as u32,
            y: rect.y as u32,
            width: rect.width as u32,
            height: rect.height as u32,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Detection {
    pub uuid: Uuid,
    pub bbox: BBox,
    pub confidence: Option<f32>,
}

impl Detection {
    pub fn new(uuid: Uuid, bbox: BBox, confidence: Option<f32>) -> Self {
        Detection {
            uuid,
            bbox,
            confidence,
        }
    }

    pub fn from_face(face: rust_faces::Face) -> Self {
        Detection {
            uuid: Uuid::new_v4(),
            bbox: BBox::from_rect(face.rect),
            confidence: Some(face.confidence),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DetectionResult {
    pub uuid: Uuid,
    pub path: PathBuf,
    pub timestamp: DateTime<Utc>,
    pub detections: Vec<Detection>,
}

impl DetectionResult {
    pub fn new(path: PathBuf, detections: Vec<Detection>) -> Self {
        DetectionResult {
            uuid: Uuid::new_v4(),
            path,
            timestamp: Utc::now(),
            detections,
        }
    }

    pub fn from_faces(path: PathBuf, faces: Vec<rust_faces::Face>) -> Self {
        let detections: Vec<Detection> = faces.iter().map(|face| Detection::from_face(face.clone())).collect();
        DetectionResult {
            uuid: Uuid::new_v4(),
            path,
            timestamp: Utc::now(),
            detections,
        }
    }
}