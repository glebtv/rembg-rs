use crate::clean_sticker_border::clean_sticker_border;
use crate::error::RembgError;
use crate::manager::ModelManager;
use crate::options::RemovalOptions;
use crate::result::RemovalResult;
use image::{DynamicImage, GenericImageView, GrayImage, Luma, Rgba, RgbaImage};
use ndarray::{Array4, Axis};

pub fn rembg(
    manager: &mut ModelManager,
    image: DynamicImage,
    options: &RemovalOptions,
) -> Result<RemovalResult, RembgError> {
    let (original_width, original_height) = image.dimensions();

    let preprocessed = {
        // Convert to RGB if not already
        let rgb_img = image.to_rgb8();

        // Resize image
        let (target_width, target_height) = manager.input_dimensions();
        let resized = image::imageops::resize(
            &rgb_img,
            target_width,
            target_height,
            image::imageops::FilterType::Lanczos3,
        );

        // Convert to normalized float array with shape [1, 3, height, width]
        let mut array = Array4::<f32>::zeros((1, 3, target_height as usize, target_width as usize));

        for (x, y, pixel) in resized.enumerate_pixels() {
            let [r, g, b] = pixel.0;

            // Normalize to [0, 1] and then to standardized range for model
            array[[0, 0, y as usize, x as usize]] = (r as f32 / 255.0 - 0.485) / 0.229;
            array[[0, 1, y as usize, x as usize]] = (g as f32 / 255.0 - 0.456) / 0.224;
            array[[0, 2, y as usize, x as usize]] = (b as f32 / 255.0 - 0.406) / 0.225;
        }

        array
    };

    // Run model inference
    let mask_output: Array4<f32> = manager.run_inference(&preprocessed)?;

    // Postprocess the mask
    let mask = {
        let temp_axis = mask_output.index_axis(Axis(0), 0);
        let mask_data = temp_axis.index_axis(Axis(0), 0);
        let (model_height, model_width) = mask_data.dim();

        let min = mask_data.iter().copied().fold(f32::INFINITY, f32::min);
        let max = mask_data.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let apply_sigmoid = min < 0.0 || max > 1.0;
        let transformed_min = if apply_sigmoid { sigmoid(min) } else { min };
        let transformed_max = if apply_sigmoid { sigmoid(max) } else { max };
        let range = (transformed_max - transformed_min).max(f32::EPSILON);

        let mut gray = GrayImage::new(model_width as u32, model_height as u32);
        for (x, y, pixel) in gray.enumerate_pixels_mut() {
            let value = mask_data[[y as usize, x as usize]];
            let value = if apply_sigmoid { sigmoid(value) } else { value };
            *pixel = Luma([(((value - transformed_min) / range) * 255.0).clamp(0.0, 255.0) as u8]);
        }

        image::imageops::resize(
            &gray,
            original_width,
            original_height,
            image::imageops::FilterType::Lanczos3,
        )
    };

    // Apply mask to original image
    let result_image = {
        // Convert input image to RGBA
        let rgba_img = image.to_rgba8();
        let (width, height) = rgba_img.dimensions();

        if mask_output.ndim() != 4 {
            return Err(RembgError::PreprocessingError(format!(
                "Unexpected mask shape: {:?}",
                mask_output.shape()
            )));
        }

        let mut result = RgbaImage::new(width, height);
        let thr_u8 = options.threshold;
        let thr_f = thr_u8 as f32;

        let smooth_scale = if thr_u8 < 255 {
            Some(255.0 / (255.0 - thr_f))
        } else {
            None
        };

        for (x, y, src) in rgba_img.enumerate_pixels() {
            let mask_value = mask.get_pixel(x, y).0[0];

            let alpha: u8 = if options.binary {
                if mask_value >= thr_u8 { 255 } else { 0 }
            } else {
                match smooth_scale {
                    Some(scale) => {
                        let mv = mask_value as f32;
                        ((mv - thr_f) * scale).clamp(0.0, 255.0).round() as u8
                    }
                    None => {
                        if mask_value == 255 {
                            255
                        } else {
                            0
                        }
                    }
                }
            };

            let alpha = ((alpha as u16 * src.0[3] as u16 + 127) / 255) as u8;
            result.put_pixel(x, y, Rgba([src.0[0], src.0[1], src.0[2], alpha]));
        }

        if options.sticker {
            result = clean_sticker_border(&result);
        }

        result
    };

    Ok(RemovalResult {
        image: result_image,
        mask,
    })
}

#[inline]
fn sigmoid(value: f32) -> f32 {
    1.0 / (1.0 + (-value).exp())
}
