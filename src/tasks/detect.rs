use std::path::Path;

use rayon::prelude::*;
use rust_faces::{
    BlazeFaceParams, FaceDetection, FaceDetectorBuilder, InferParams, Provider
};
use rust_faces::ToArray3;
use image::DynamicImage;
use crate::schema::detection_result::DetectionResult;
use crate::schema::images::Image;

fn load_image(path: &Path) -> Result<DynamicImage, String> {
    image::open(path).map_err(|_| format!("Can't open test image at {:?}", path))
}

fn compute_scale_factor(original_width: u32, original_height: u32, max_size: u32) -> f32 {
    let scale_factor = if original_width > original_height {
        max_size as f32 / original_width as f32
    } else {
        max_size as f32 / original_height as f32
    };
    scale_factor
}

fn scale_dimensions(original_width: u32, original_height: u32, max_size: u32) -> (u32, u32) {
    let scale_factor = compute_scale_factor(original_width, original_height, max_size);
    (
        (original_width as f32 * scale_factor).round() as u32,
        (original_height as f32 * scale_factor).round() as u32,
    )
}

fn resize_image(img: &DynamicImage, max_size: u32) -> DynamicImage {
    let (new_width, new_height) = scale_dimensions(img.width(), img.height(), max_size);
    img.resize_exact(new_width, new_height, image::imageops::FilterType::Nearest)
        .into()
}


fn detect_faces(files: &Vec<Image>) -> Result<Vec<DetectionResult>, String> {
    log::info!("Init detector...");
    let face_detector = FaceDetectorBuilder::new(FaceDetection::BlazeFace320(BlazeFaceParams::default()))
        .download()
        .infer_params(InferParams {
            provider: Provider::OrtCoreMl,
            intra_threads: Some(12),
            ..Default::default()
        })
        .build()
        .map_err(|e| format!("Failed to load face detector: {}", e))?;

    log::info!("Applying face detection to {} images...", files.len());

    let results = files.par_iter().map(|file| {
        let path = file.path.clone();
        match load_image(&path) {
            Ok(img) => {
                match face_detector.detect(
                    resize_image(&img, 512)
                        .to_rgb8()
                        .into_array3()
                        .view()
                        .into_dyn(),
                ) {
                    Ok(faces) => {
                        Ok(
                            DetectionResult::from_faces(path, faces)
                        )
                    }
                    Err(e) => Err(format!("Error detecting faces in image at {:?}: {}", path, e)),
                }
            }
            Err(e) => Err(e),
        }
    }).collect::<Result<Vec<_>, String>>();

    results
}

pub fn run(images: &Vec<Image>) -> Result<Vec<DetectionResult>, String> {
    detect_faces(images)
}