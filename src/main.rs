use clap::Parser;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use rayon::prelude::*;
use log::{info, error};
use env_logger;
use image::{GenericImageView,GenericImage, Rgb, RgbImage, ImageBuffer};
use dlib_face_recognition::*;
use show_image::{create_window,WindowOptions, ImageView, ImageInfo};
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(name = "facia", version = "0.1.0", author = "Lukas Sanner")]
#[command(about = "A tool to detect and classify faces in images")]
struct Args {
    #[arg(required = true, help = "The path to the directory containing the images")]
    directories: Vec<PathBuf>,

    #[arg(required = true, help = "Number of used threads")]
    num_threads: Option<usize>,
}

fn is_valid_image_file(path: &Path) -> bool {
    let valid_extensions = ["jpg", "jpeg", "JPG", "JPEG", "heic", "HEIC"];
    path.extension()
        .and_then(|ext| ext.to_str())
        .map_or(false, |ext| valid_extensions.contains(&ext))
}

fn scan_directory(dir: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for entry in WalkDir::new(dir) {
        match entry {
            Ok(entry) => {
                let path = entry.path().to_path_buf(); 
                if path.is_dir() {
                    info!("Found directory: {:?}", path);
                } else if path.is_file() && is_valid_image_file(&path) {
                    paths.push(path);
                }
            }
            Err(e) => error!("Error reading directory entry: {}", e),
        }
    }
    paths
}

fn read_and_rescale_image(path: &str, max_size: u32) -> RgbImage {
    let img = image::open(path).expect("Failed to open image");
    let (width, height) = img.dimensions();
    let scale_factor = if width > height {
        max_size as f32 / width as f32
    } else {
        max_size as f32 / height as f32
    };
    let new_width = (width as f32 * scale_factor).round() as u32;
    let new_height = (height as f32 * scale_factor).round() as u32;
    img.resize_exact(new_width, new_height, image::imageops::FilterType::Nearest).into()
}

fn draw_rectangle(rgb_image: &mut RgbImage, rect: &Rectangle, colour: Rgb<u8>) {
    for x in rect.left..rect.right {
        rgb_image.put_pixel(x as u32, rect.top as u32, colour);
        rgb_image.put_pixel(x as u32, rect.bottom as u32, colour);
    }
    for y in rect.top..rect.bottom {
        rgb_image.put_pixel(rect.left as u32, y as u32, colour);
        rgb_image.put_pixel(rect.right as u32, y as u32, colour);
    }
}


fn create_image_grid(images: Vec<RgbImage>, max_size: u32) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    // Calculate the number of rows and columns
    let n = images.len();
    let (rows, cols) = compute_grid_dimensions(n);

    // Calculate the dimensions of the final image
    let grid_width = cols as u32 * max_size;
    let grid_height = rows as u32 * max_size;

    // Create a new blank image with the computed dimensions
    let mut grid_image = RgbImage::new(grid_width, grid_height);

    // Place each image into the grid
    for (index, img) in images.iter().enumerate() {
        // Determine the position in the grid
        let row = index / cols;
        let col = index % cols;

        // Calculate the top-left position where the image should be placed
        let x_offset = col as u32 * max_size;
        let y_offset = row as u32 * max_size;

        // Copy each image into the appropriate position on the grid
        grid_image.copy_from(img, x_offset.into(), y_offset.into()).unwrap();
    }

    grid_image
}

// Helper function to compute the grid dimensions (as defined in the previous response)
fn compute_grid_dimensions(n: usize) -> (usize, usize) {
    let mut cols = (n as f64).sqrt().ceil() as usize;
    let mut rows = (n + cols - 1) / cols;

    while rows * cols >= n && (rows - 1) * cols >= n {
        rows -= 1;
    }
    while rows * cols < n {
        cols += 1;
    }

    (rows, cols)
}

#[show_image::main]
fn main() -> Result<(), Box<dyn std::error::Error>>{    
    let start_time = Instant::now();

    let window = create_window("detected faces", WindowOptions::new()).unwrap();
    env_logger::init();
    let args = Args::parse();
    let dirs = args.directories;
    let num_threads = args.num_threads.unwrap();

    let image_paths: Vec<PathBuf> = dirs.par_iter().flat_map(|dir| {
        scan_directory(&dir)
    }).collect();

    let image_paths_len = image_paths.len();
    info!("Found {} images in all directories", image_paths_len);

    info!("Creating face detector");
    let face_detection_model = dlib_face_recognition::FaceDetector::default();

    let green = Rgb([0, 255, 0]);
    let max_size = 512;
    
    info!("Processing images");
    
    let pool = threadpool::ThreadPool::new(num_threads);
    use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

    let num_faces = Arc::new(AtomicUsize::new(0));
    
    for batch in image_paths.chunks(num_threads) {
        let face_detection_model = face_detection_model.clone();
        let window = window.clone();
        let batch = batch.to_vec();
        
        // Clone the Arc to pass into the closure
        let num_faces = Arc::clone(&num_faces);
    
        pool.execute(move || {
            let mut batch_images: Vec<RgbImage> = Vec::new();
            for path in batch {
                let mut resized_image = read_and_rescale_image(&path.as_os_str().to_str().unwrap(), max_size);
                let matrix = ImageMatrix::from_image(&resized_image);
                let face_locations = face_detection_model.face_locations(&matrix);
                
                // Atomically add the face count from this image
                num_faces.fetch_add(face_locations.len(), Ordering::Relaxed);
    
                for r in face_locations.iter() {
                    draw_rectangle(&mut resized_image, &r, green);
                }
    
                batch_images.push(resized_image);
            }
    
            let combined_image = create_image_grid(batch_images, max_size);
            let image_info = ImageInfo::rgb8(combined_image.width(), combined_image.height());
            let image_view = ImageView::new(image_info, combined_image.as_raw());
        
            window.set_image("image", &image_view).unwrap();
        });
    }
    
    pool.join();
    let elapsed_time = start_time.elapsed();
    println!("Number of faces detected: {} in {} images", num_faces.load(Ordering::Relaxed), image_paths_len);
    println!("Total execution time: {:?}", elapsed_time);
    Ok(())

    
}