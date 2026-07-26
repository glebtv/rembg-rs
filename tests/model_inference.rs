use image::GenericImageView;
use rembg_rs::{manager::ModelManager, options::RemovalOptions, rembg::rembg};
use std::path::{Path, PathBuf};

#[test]
#[ignore = "requires downloaded ONNX models"]
fn all_downloaded_models_process_all_test_images() {
    let model_dir = std::env::var("MODEL_DIR").unwrap_or_else(|_| "models".to_string());
    let image_dir = std::env::var("TEST_IMAGE_DIR").unwrap_or_else(|_| "test-images".to_string());
    let result_dir = std::env::var_os("RESULT_DIR").map(PathBuf::from);
    let models = files_with_extension(Path::new(&model_dir), "onnx");
    let images = image_files(Path::new(&image_dir));

    assert!(!models.is_empty(), "no ONNX models found in models/");
    assert!(!images.is_empty(), "no test images found in test-images/");

    let mut failures = Vec::new();
    for model_path in &models {
        eprintln!("loading {}", model_path.display());
        let mut manager = match ModelManager::from_file(model_path) {
            Ok(manager) => manager,
            Err(error) => {
                failures.push(format!("{}: load failed: {error}", model_path.display()));
                continue;
            }
        };

        for image_path in &images {
            eprintln!("  processing {}", image_path.display());
            let input = match image::open(image_path) {
                Ok(image) => image,
                Err(error) => {
                    failures.push(format!(
                        "{}: image load failed: {error}",
                        image_path.display()
                    ));
                    continue;
                }
            };
            let dimensions = input.dimensions();

            match rembg(&mut manager, input, &RemovalOptions::default()) {
                Ok(result) => {
                    if result.image().dimensions() != dimensions {
                        failures.push(format!(
                            "{} + {}: result dimensions mismatch",
                            model_path.display(),
                            image_path.display()
                        ));
                    }
                    if result.mask().dimensions() != dimensions {
                        failures.push(format!(
                            "{} + {}: mask dimensions mismatch",
                            model_path.display(),
                            image_path.display()
                        ));
                    }

                    let (mask_min, mask_max) = result
                        .mask()
                        .pixels()
                        .map(|pixel| pixel.0[0])
                        .fold((u8::MAX, u8::MIN), |(min, max), value| {
                            (min.min(value), max.max(value))
                        });
                    if mask_min == mask_max {
                        failures.push(format!(
                            "{} + {}: mask is constant ({mask_min})",
                            model_path.display(),
                            image_path.display()
                        ));
                    }

                    if let Some(result_dir) = &result_dir {
                        let model_name = model_path
                            .file_stem()
                            .and_then(|value| value.to_str())
                            .unwrap_or("model");
                        let image_name = image_path
                            .file_stem()
                            .and_then(|value| value.to_str())
                            .unwrap_or("image");
                        let model_result_dir = result_dir.join(model_name);
                        if let Err(error) = std::fs::create_dir_all(&model_result_dir) {
                            failures.push(format!(
                                "{}: result directory creation failed: {error}",
                                model_result_dir.display()
                            ));
                            continue;
                        }

                        let output_path = model_result_dir.join(format!("{image_name}.png"));
                        let mask_path = model_result_dir.join(format!("{image_name}_mask.png"));
                        if let Err(error) = result.image().save(&output_path) {
                            failures
                                .push(format!("{}: save failed: {error}", output_path.display()));
                        }
                        if let Err(error) = result.mask().save(&mask_path) {
                            failures.push(format!("{}: save failed: {error}", mask_path.display()));
                        }
                    }
                }
                Err(error) => failures.push(format!(
                    "{} + {}: inference failed: {error}",
                    model_path.display(),
                    image_path.display()
                )),
            }
        }
    }

    assert!(
        failures.is_empty(),
        "model failures:\n{}",
        failures.join("\n")
    );
}

fn files_with_extension(directory: &Path, extension: &str) -> Vec<PathBuf> {
    let mut paths: Vec<_> = std::fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", directory.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some(extension))
        .collect();
    paths.sort();
    paths
}

fn image_files(directory: &Path) -> Vec<PathBuf> {
    let mut paths: Vec<_> = std::fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", directory.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            matches!(
                path.extension().and_then(|value| value.to_str()).map(str::to_ascii_lowercase),
                Some(extension) if matches!(extension.as_str(), "jpg" | "jpeg" | "png" | "webp")
            )
        })
        .collect();
    paths.sort();
    paths
}
