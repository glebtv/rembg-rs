use clap::Parser;
use image::{DynamicImage, open};
use rembg_rs::cli::cli::{Args, Commands, RemoveBgArgs};
use rembg_rs::compress_png::compress_png;
use rembg_rs::manager::ModelManager;
use rembg_rs::options::RemovalOptionsBuilder;
use rembg_rs::rembg::rembg;
use std::path::Path;
use std::process;

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::RemoveBg(args) => run_remove_bg(&args),

        #[cfg(feature = "download")]
        Commands::DownloadModels(args) => {
            if let Err(e) = rembg_rs::models::download::download_models(&args) {
                eprintln!("❌ {}", e);
                process::exit(1);
            }
        }
    }
}

fn run_remove_bg(args: &RemoveBgArgs) {
    println!("🎨 rembg-rs - Background Removal Tool");
    println!("Input: {:?}", args.input);
    println!("Output: {:?}", args.output);
    println!("Model: {}", args.model);
    println!();

    let mut manager = match ModelManager::from_file(Path::new(&args.model)) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("❌ Failed to load model: {}", e);
            process::exit(1);
        }
    };

    println!("✅ Model loaded\n");

    println!("📂 Loading image...");
    let img = match open(&args.input) {
        Ok(img) => img,
        Err(e) => {
            eprintln!("❌ Failed to load image: {}", e);
            process::exit(1);
        }
    };

    let options = RemovalOptionsBuilder::default()
        .threshold(args.threshold)
        .binary(args.binary)
        .sticker(args.sticker)
        .build()
        .unwrap();

    println!("🖼️  Processing image...");

    let result = match rembg(&mut manager, img, &options) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("❌ Error: {}", e);
            process::exit(1);
        }
    };

    println!("💾 Saving result...");
    let result_img: DynamicImage = DynamicImage::ImageRgba8(result.image().clone());
    if !args.no_compress {
        match compress_png(&result_img) {
            Ok(bytes) => match std::fs::write(&args.output, bytes) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("❌ Writing failed: {e}");
                    process::exit(1);
                }
            },
            Err(e) => {
                eprintln!("⚠️  Compression failed ({e}); saving uncompressed PNG");
                if let Err(save_error) = result_img.save(&args.output) {
                    eprintln!("❌ Failed to save result: {save_error}");
                    process::exit(1);
                }
            }
        }
    } else if let Err(e) = result_img.save(&args.output) {
        eprintln!("❌ Failed to save result: {}", e);
        process::exit(1);
    }

    if args.save_mask {
        let mask_path = generate_mask_path(&args.output);
        println!("🎭 Saving mask to: {:?}", mask_path);

        let mask_img = result.mask();
        if let Err(e) = mask_img.save(&mask_path) {
            eprintln!("⚠️  Failed to save mask: {}", e);
        }
    }

    println!();
    println!("✅ Background removed successfully!");
    println!("Output saved to: {:?}", args.output);
    if args.save_mask {
        println!("🎭 Mask saved alongside output");
    }
}

fn generate_mask_path(output_path: &Path) -> std::path::PathBuf {
    let file_stem = output_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    let extension = output_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("png");

    let parent = output_path.parent().unwrap_or(Path::new("."));

    parent.join(format!("{}_mask.{}", file_stem, extension))
}
