//! Model download functionality with progress reporting and verification

use crate::cli::cli::DownloadModelsArgs;
use crate::models::models::{ALL_MODELS, ChecksumType, ModelDef};
use md5::{Digest, Md5};
use sha2::Sha256;
use std::fs::{File, metadata};
use std::io::{Read, Write};
use std::path::Path;
use std::time::Instant;

/// Download requested models
pub fn download_models(args: &DownloadModelsArgs) -> anyhow::Result<()> {
    let models_dir = &args.dir;
    let names = parse_names(&args.names)?;

    println!("📦 rembg-rs Model Downloader");
    println!("Target directory: {:?}", models_dir);
    println!("Models to download: {:?}", names);
    println!();

    std::fs::create_dir_all(models_dir)?;

    let mut total_size: u64 = 0;
    let mut success_count: usize = 0;
    let mut skip_count: usize = 0;

    for (i, model) in names.iter().enumerate() {
        let def = ALL_MODELS
            .iter()
            .find(|m| m.name == model)
            .ok_or_else(|| anyhow::anyhow!("Unknown model: {}", model))?;

        let is_last = i == names.len() - 1;
        let progress = if names.len() > 1 {
            format!("[{}/{}] ", i + 1, names.len())
        } else {
            String::new()
        };

        println!("{}Model: {} - {}", progress, def.name, def.description);

        let dest_path = models_dir.join(def.filename);

        if !args.force && dest_path.exists() {
            let existing_size = metadata(&dest_path).map(|m| m.len()).unwrap_or(0);
            let verified = if args.skip_verify {
                true
            } else {
                verify_file(&dest_path, def)
            };

            if verified {
                println!(
                    "  ⏭️  Already exists (verified, {})",
                    ModelDef::format_size(existing_size)
                );
                skip_count += 1;
                if !is_last {
                    println!();
                }
                continue;
            } else {
                println!("  ⚠️  Checksum mismatch, re-downloading...");
            }
        }

        match download_file(def, &dest_path, args.skip_verify) {
            Ok(result) => {
                println!(
                    "  ✅ Downloaded {} in {:.1}s (verified: {})",
                    ModelDef::format_size(result.size_bytes),
                    result.duration_secs,
                    result.verified
                );
                total_size += result.size_bytes;
                success_count += 1;
            }
            Err(e) => {
                eprintln!("  ❌ Failed: {}", e);
            }
        }

        if !is_last {
            println!();
        }
    }

    println!();
    println!("📊 Summary:");
    println!("  Downloaded: {}", success_count);
    println!("  Skipped: {}", skip_count);
    println!("  Total size: {}", ModelDef::format_size(total_size));
    println!("  Location: {:?}", models_dir);

    Ok(())
}

/// Parse model names from comma-separated string
fn parse_names(input: &str) -> anyhow::Result<Vec<String>> {
    let input = input.trim();

    if input == "all" {
        Ok(ALL_MODELS.iter().map(|m| m.name.to_string()).collect())
    } else {
        let names: Vec<String> = input
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if names.is_empty() {
            anyhow::bail!("No model names provided");
        }

        Ok(names)
    }
}

/// Download a single model file with progress reporting
fn download_file(
    def: &ModelDef,
    dest_path: &Path,
    skip_verify: bool,
) -> anyhow::Result<crate::models::models::DownloadResult> {
    let start = Instant::now();

    println!("  ⬇️  Downloading from {}", def.url);

    let response = ureq::get(def.url).call()?;

    let status = response.status();
    if status.is_client_error() || status.is_server_error() {
        anyhow::bail!(
            "HTTP {} {}",
            status.as_u16(),
            status.canonical_reason().unwrap_or("")
        );
    }

    let body = response.into_body();
    let total_bytes: u64 = body.content_length().unwrap_or(0);

    let mut downloaded: u64 = 0;
    let mut file = File::create(dest_path)?;
    let mut buffer = [0u8; 65536];

    let mut reader = body.into_reader();

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        file.write_all(&buffer[..bytes_read])?;
        downloaded += bytes_read as u64;

        let progress = if total_bytes > 0 {
            format!(
                "{:>6.1}% ",
                (downloaded as f64 / total_bytes as f64) * 100.0
            )
        } else {
            format!("{:>8} ", ModelDef::format_size(downloaded))
        };

        let total_str = if total_bytes > 0 {
            ModelDef::format_size(total_bytes)
        } else {
            String::from("...")
        };
        print!(
            "\r  {}{} / {}",
            progress,
            ModelDef::format_size(downloaded),
            total_str
        );
        let _ = std::io::stdout().flush();
    }

    println!();

    let duration = start.elapsed().as_secs_f32();
    let verified = if skip_verify {
        println!("  ⚠️  Checksum verification skipped");
        false
    } else {
        verify_file(dest_path, def)
    };

    Ok(crate::models::models::DownloadResult {
        name: def.name.to_string(),
        path: dest_path.to_string_lossy().to_string(),
        size_bytes: downloaded,
        duration_secs: duration,
        verified,
    })
}

/// Verify file checksum
fn verify_file(path: &Path, def: &ModelDef) -> bool {
    println!("  🔍 Verifying checksum...");

    match def.checksum_type {
        ChecksumType::Md5 => {
            let mut hasher = Md5::new();
            let mut file = match File::open(path) {
                Ok(f) => f,
                Err(_) => {
                    println!("  ❌ Cannot open file for verification");
                    return false;
                }
            };

            let mut buffer = [0u8; 65536];
            loop {
                let bytes_read = match file.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => n,
                    Err(_) => {
                        println!("  ❌ Read error during verification");
                        return false;
                    }
                };
                hasher.update(&buffer[..bytes_read]);
            }

            let result = format!("{:x}", hasher.finalize());
            if result == def.checksum {
                println!("  ✅ MD5 verified");
                true
            } else {
                println!(
                    "  ❌ MD5 mismatch: expected {}, got {}",
                    def.checksum, result
                );
                false
            }
        }
        ChecksumType::Sha256 => {
            let mut hasher = Sha256::new();
            let mut file = match File::open(path) {
                Ok(f) => f,
                Err(_) => {
                    println!("  ❌ Cannot open file for verification");
                    return false;
                }
            };

            let mut buffer = [0u8; 65536];
            loop {
                let bytes_read = match file.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => n,
                    Err(_) => {
                        println!("  ❌ Read error during verification");
                        return false;
                    }
                };
                hasher.update(&buffer[..bytes_read]);
            }

            let result = format!("{:x}", hasher.finalize());
            if result == def.checksum {
                println!("  ✅ SHA256 verified");
                true
            } else {
                println!("  ❌ SHA256 mismatch");
                false
            }
        }
    }
}
