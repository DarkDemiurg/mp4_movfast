use glob::glob;
use std::env;
use std::fs;
use std::io;
use std::io::ErrorKind;
use std::path::Path;
use std::process::{Command, ExitCode};

fn mp4_optimize(filename: &Path) -> Result<bool, std::io::Error> {
    if !filename.exists() {
        return Err(io::Error::new(
            ErrorKind::InvalidFilename,
            "Source file not exist",
        ));
    }

    println!("Try optimize file: {}", filename.display());

    let new_filename = filename.with_extension("new.mp4");

    let output = Command::new("ffmpeg")
        .arg("-hide_banner")
        .arg("-i")
        .arg(&filename)
        .arg("-c")
        .arg("copy")
        .arg("-movflags")
        .arg("+faststart")
        .arg("-y")
        .arg(&new_filename)
        .output()?;

    if output.status.success() {
        println!("  File optimized: {}", filename.display());
        match fs::remove_file(&filename) {
            Ok(_) => {
                println!(
                    "    Source file removed successfully: {}",
                    new_filename.display()
                );
                match fs::rename(&new_filename, filename) {
                    Ok(_) => {
                        println!(
                            "      New file renamed successfully: {}",
                            new_filename.display()
                        );
                        Ok(true)
                    }
                    Err(e) => {
                        eprintln!(
                            "Error renaming file {} to {}: {}",
                            new_filename.display(),
                            filename.display(),
                            e
                        );
                        Err(e)
                    }
                }
            }
            Err(e) => {
                eprintln!("Error removing file {}: {}", new_filename.display(), e);
                Err(e)
            }
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Command failed with error:\n{}", stderr);

        Err(io::Error::new(
            ErrorKind::Other,
            format!("FFmpeg failed with error: {}", stderr),
        ))
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        println!("Program name: {}", args[0]);
        println!("Arguments:");
        for (i, arg) in args.iter().skip(1).enumerate() {
            println!("  Argument {}: {}", i + 1, arg);
        }
    } else {
        eprintln!("No command-line arguments provided (except program name).");
        return ExitCode::from(1);
    }

    let work_path = Path::new(&args[1]);

    if !work_path.exists() {
        eprintln!("Target path not found: {}", work_path.display());
        return ExitCode::from(2);
    }

    if work_path.is_file() {
        return optimize_one_file(work_path).unwrap();
    }

    if !work_path.is_dir() {
        eprintln!(
            "Target path is not file or directory: {}",
            work_path.display()
        );
        return ExitCode::from(3);
    }

    let work_path = work_path.join("**").join("*.mp4");

    let pattern = work_path.to_string_lossy();

    let mut i: usize = 0;
    let files_count = match glob(&pattern) {
        Ok(paths) => paths.into_iter().collect::<Vec<_>>().len(),
        Err(e) => {
            eprintln!("Error reading glob directory: {}", e);
            return ExitCode::from(2);
        }
    };

    for entry in glob(&pattern).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                if path.is_file() && path.extension().unwrap_or_default() == "mp4" {
                    i += 1;
                    match mp4_optimize(&path) {
                        Ok(_) => {
                            println!(
                                "[{:0>3}/{:0>3}] File optimized successfully: {}",
                                i,
                                files_count,
                                path.display()
                            )
                        }
                        Err(e) => {
                            eprintln!(
                                "[{:0>3}/{:0>3}] Error optimize file {}: {}",
                                i,
                                files_count,
                                path.display(),
                                e
                            )
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading glob directory: {}", e);
                return ExitCode::from(2);
            }
        }
    }

    return ExitCode::SUCCESS;
}

fn optimize_one_file(work_path: &Path) -> Option<ExitCode> {
    if work_path.extension().unwrap_or_default() == "mp4" {
        match mp4_optimize(&work_path) {
            Ok(_) => {
                println!("File optimized successfully: {}", work_path.display());
                return Some(ExitCode::SUCCESS);
            }
            Err(e) => {
                eprintln!("Error optimize file {}: {}", work_path.display(), e);
                return Some(ExitCode::from(4));
            }
        }
    } else {
        eprintln!("Only MP4 file supported");
        return Some(ExitCode::from(5));
    }
}
