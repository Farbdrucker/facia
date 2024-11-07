# facia
Personal project to learn rust 

## Idea
This project aims to scan my image collection for people and create a DB, such that I know who is present in which photos

- scan the given directory and its sub directories for images (`{"jpg"|"jpeg"|"JPG"|"JPEG"|"heic"|"HEIC"}`) (note, `heic` images not yet testd)
- apply `dlib_face_recognition` and detect faces
- render images with detected faces

### TODO
- apply face embedding to cropped faces
- create DB with image overview, people, face embeddings
- hash images to avoid duplicate work

## Installation
```
git clone https://github.com/Farbdrucker/facia.git
cd facia
cargo build --release
```

Building with `--release` makes the programm _go wroooom_

## Run
```
cargo run --realse /path/to/your/directories/with/images "${NUM_THREADS}"
```
