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

    let new_filename = filename.with_extension("new.mp4");

    let output = Command::new("ffmpeg")
        .arg("-hide_banner")
        .arg("-i")
        .arg(&filename)
        .arg("-c")
        .arg("copy")
        .arg("-movflags")
        .arg("faststart")
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
                        println!("      New file renamed successfully: {}", new_filename.display());
                        Ok(true)
                    }
                    Err(e) => {
                        eprintln!("Error renaming file {} to {}: {}", new_filename.display(), filename.display(), e);
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
            format!("FFmpeg failed with error: {}", stderr)
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

    match fs::read_dir(work_path) {
        Ok(entries) => {
            for entry_result in entries {
                match entry_result {
                    Ok(entry) => {
                        let path = entry.path();
                        if path.is_file() && path.extension().unwrap_or_default() == "mp4" {
                            println!("Try optimize file: {}", path.display());
                            match mp4_optimize(&path) {
                                Ok(_) => {
                                    println!("File optimized successfully: {}", path.display())
                                }
                                Err(e) => {
                                    eprintln!("Error optimize file {}: {}", path.display(), e)
                                }
                            }
                        }
                        if path.is_dir() {
                            println!("Sub dir found: {}", path.display());
                        }
                    }
                    Err(e) => eprintln!("Error reading directory entry: {}", e),
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading directory: {}", e);
            return ExitCode::from(2);
        }
    }

    return ExitCode::SUCCESS;
}
