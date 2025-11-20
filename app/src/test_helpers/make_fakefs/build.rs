use std::env;
use std::path::Path;
use std::process::exit;

use fileZoom::building::make_fakefs_lib;

pub fn build_image() {
    let current = env::current_dir().expect("Failed to get current dir");
    match make_fakefs_lib::build_image_with_fixtures(None, &current) {
        Ok(()) => println!("Docker image 'filezoom-fakefs' built successfully."),
        Err(e) => {
            eprintln!("Failed to build image: {}", e);
            exit(1);
        }
    }
}

pub fn build_image_with_fixtures(fixtures: Option<&Path>) {
    let current = env::current_dir().expect("Failed to get current dir");
    match make_fakefs_lib::build_image_with_fixtures(fixtures, &current) {
        Ok(()) => {
            println!("Docker image 'filezoom-fakefs' built successfully (using temp context).")
        }
        Err(e) => {
            eprintln!("Failed to build image: {}", e);
            exit(1);
        }
    }
}
