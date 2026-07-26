use axum::{
    Json, Router,
    body::Body,
    extract::{Multipart, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use std::path::Path;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tracing::error;

use crate::compress_png::compress_png;
use crate::error::RembgError;
use crate::options::RemovalOptions;
use crate::rembg::rembg;
use crate::server::state::AppState;
use crate::server::types::{ConfigResponse, ErrorResponse, RemoveBgRequest, RemoveBgResponse};

pub fn create_router(app_state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
        .allow_headers(Any)
        .allow_origin(Any);

    Router::new()
        .route("/healthz", axum::routing::get(healthz))
        .route("/api/auth/validate", axum::routing::get(validate_api_key))
        .route("/api/remove-bg", axum::routing::post(remove_bg))
        .route("/api/config", axum::routing::get(get_config))
        .fallback_service(
            ServeDir::new("web/build").fallback(ServeFile::new("web/build/index.html")),
        )
        .layer(cors)
        .with_state(app_state)
}

#[axum::debug_handler]
pub async fn healthz() -> &'static str {
    "OK"
}

#[axum::debug_handler]
pub async fn get_config(state: State<AppState>) -> Json<ConfigResponse> {
    Json(ConfigResponse {
        model_path: state.config.model_path.to_string_lossy().to_string(),
        host: state.config.host.clone(),
        port: state.config.port,
        auth_required: state.config.auth_required,
    })
}

#[axum::debug_handler]
pub async fn validate_api_key(
    state: State<AppState>,
    headers: HeaderMap,
) -> Result<&'static str, AppError> {
    require_api_key(&state, &headers)?;
    Ok("OK")
}

#[axum::debug_handler]
pub async fn remove_bg(
    state: State<AppState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Response<Body>, AppError> {
    require_api_key(&state, &headers)?;

    let mut file_bytes: Option<Vec<u8>> = None;
    let mut file_name = "input.png".to_string();
    let mut request_data = RemoveBgRequest {
        threshold: None,
        binary: None,
        sticker: None,
        compress: None,
        format: None,
        include_mask: None,
    };

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        error!("Multipart error: {}", e);
        AppError::BadRequest("Invalid multipart form data".to_string())
    })? {
        let name = field.name().map(|s| s.to_string());
        let filename = field.file_name().map(|s| s.to_string());

        let value = field.bytes().await.map_err(|e| {
            error!("Field read error: {}", e);
            AppError::BadRequest("Failed to read field".to_string())
        })?;

        match name.as_deref() {
            Some("image" | "file") => {
                file_bytes = Some(value.to_vec());
                if let Some(fname) = filename {
                    file_name = fname;
                }
            }
            Some("threshold") => {
                request_data.threshold = std::str::from_utf8(&value)
                    .ok()
                    .and_then(|s| s.parse().ok());
            }
            Some("binary") => {
                request_data.binary = std::str::from_utf8(&value)
                    .ok()
                    .and_then(|s| s.parse().ok());
            }
            Some("sticker") => {
                request_data.sticker = std::str::from_utf8(&value)
                    .ok()
                    .and_then(|s| s.parse().ok());
            }
            Some("compress") => {
                request_data.compress = std::str::from_utf8(&value)
                    .ok()
                    .and_then(|s| s.parse().ok());
            }
            Some("format") => {
                request_data.format = std::str::from_utf8(&value).ok().map(|s| s.to_lowercase());
            }
            Some("includeMask" | "include_mask") => {
                request_data.include_mask = std::str::from_utf8(&value)
                    .ok()
                    .and_then(|s| s.parse().ok());
            }
            _ => {}
        }
    }

    let file_bytes =
        file_bytes.ok_or_else(|| AppError::BadRequest("No image file provided".to_string()))?;

    let img = image::load_from_memory(&file_bytes).map_err(|e| {
        error!("Image load error: {}", e);
        AppError::BadRequest("Invalid image file".to_string())
    })?;

    let options = RemovalOptions {
        threshold: request_data.threshold.unwrap_or(160),
        binary: request_data.binary.unwrap_or(false),
        sticker: request_data.sticker.unwrap_or(false),
    };

    let result = tokio::task::spawn_blocking({
        let model_guard = state.model.clone();
        move || {
            let mut model = model_guard.blocking_lock();
            rembg(&mut model, img, &options)
        }
    })
    .await
    .map_err(|e| {
        error!("Task join error: {}", e);
        AppError::InternalServerError("Processing failed".to_string())
    })?
    .map_err(|e| {
        error!("Rembg error: {}", e);
        match e {
            RembgError::InvalidInput(_) => AppError::BadRequest(e.to_string()),
            RembgError::UnsupportedFormat(_) => AppError::BadRequest(e.to_string()),
            RembgError::OnnxRuntimeNotAvailable(_) => {
                AppError::InternalServerError("Model error".to_string())
            }
            RembgError::OnnxError(_) => {
                AppError::InternalServerError("Model inference failed".to_string())
            }
            _ => AppError::InternalServerError(e.to_string()),
        }
    })?;

    let output_format = request_data.format.as_deref().unwrap_or("png");
    let compress = request_data.compress.unwrap_or(false);
    let include_mask = request_data.include_mask.unwrap_or(false);

    let (output_data, output_ext) = match output_format {
        "png" => {
            let mut buf = Vec::new();
            result
                .image()
                .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
                .map_err(|_| AppError::InternalServerError("PNG encoding failed".to_string()))?;

            let final_data = if compress {
                let dyn_img = image::load_from_memory(&buf)
                    .map_err(|_| AppError::InternalServerError("PNG decode failed".to_string()))?;
                compress_png(&dyn_img).map_err(|_| {
                    AppError::InternalServerError("PNG compression failed".to_string())
                })?
            } else {
                buf
            };
            (final_data, "png")
        }
        "jpeg" | "jpg" => {
            let mut buf = Vec::new();
            flatten_onto_white(result.image())
                .write_to(
                    &mut std::io::Cursor::new(&mut buf),
                    image::ImageFormat::Jpeg,
                )
                .map_err(|_| AppError::InternalServerError("JPEG encoding failed".to_string()))?;
            (buf, "jpg")
        }
        "webp" => {
            let mut buf = Vec::new();
            result
                .image()
                .write_to(
                    &mut std::io::Cursor::new(&mut buf),
                    image::ImageFormat::WebP,
                )
                .map_err(|_| AppError::InternalServerError("WebP encoding failed".to_string()))?;
            (buf, "webp")
        }
        _ => {
            let mut buf = Vec::new();
            result
                .image()
                .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
                .map_err(|_| AppError::InternalServerError("PNG encoding failed".to_string()))?;
            (buf, "png")
        }
    };

    let mime_type = match output_ext {
        "png" => "image/png",
        "jpg" => "image/jpeg",
        "webp" => "image/webp",
        _ => "image/png",
    };

    let output_filename = {
        let stem = Path::new(&file_name)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("output");
        format!("{}-removed.{}", stem, output_ext)
    };

    if include_mask {
        let (mask_data, mask_ext) = {
            let mut buf = Vec::new();
            result
                .mask()
                .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
                .map_err(|_| AppError::InternalServerError("Mask encoding failed".to_string()))?;
            (buf, "png")
        };

        let mask_filename = {
            let stem = Path::new(&file_name)
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("mask");
            format!("{}-mask.{}", stem, mask_ext)
        };

        let body = serde_json::to_vec(&RemoveBgResponse {
            image_data: BASE64.encode(output_data),
            image_format: mime_type.to_string(),
            filename: output_filename,
            mask_data: Some(BASE64.encode(mask_data)),
            mask_format: Some("image/png".to_string()),
            mask_filename: Some(mask_filename),
        })
        .map_err(|_| AppError::InternalServerError("Response encoding failed".to_string()))?;

        let mut response = Response::new(Body::from(body));
        response.headers_mut().insert(
            axum::http::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );
        Ok(response)
    } else {
        let mut headers = HeaderMap::new();
        headers.insert(axum::http::header::CONTENT_TYPE, mime_type.parse().unwrap());
        headers.insert(
            axum::http::header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", output_filename)
                .parse()
                .unwrap(),
        );
        let mut res = Response::new(Body::from(output_data));
        *res.headers_mut() = headers;
        Ok(res)
    }
}

fn require_api_key(state: &AppState, headers: &HeaderMap) -> Result<(), AppError> {
    authorize(state.api_key.as_deref(), headers)
}

fn authorize(expected: Option<&str>, headers: &HeaderMap) -> Result<(), AppError> {
    let Some(expected) = expected else {
        return Ok(());
    };

    let supplied = headers
        .get("x-api-key")
        .and_then(|value| value.to_str().ok())
        .or_else(|| {
            headers
                .get(axum::http::header::AUTHORIZATION)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.strip_prefix("Bearer "))
        });

    if supplied.is_some_and(|key| constant_time_eq(key.as_bytes(), expected.as_bytes())) {
        Ok(())
    } else {
        Err(AppError::Unauthorized(
            "Invalid or missing API key".to_string(),
        ))
    }
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right)
        .fold(0_u8, |difference, (left, right)| {
            difference | (left ^ right)
        })
        == 0
}

fn flatten_onto_white(image: &image::RgbaImage) -> image::RgbImage {
    image::RgbImage::from_fn(image.width(), image.height(), |x, y| {
        let pixel = image.get_pixel(x, y).0;
        let alpha = pixel[3] as u16;
        image::Rgb([
            ((pixel[0] as u16 * alpha + 255 * (255 - alpha) + 127) / 255) as u8,
            ((pixel[1] as u16 * alpha + 255 * (255 - alpha) + 127) / 255) as u8,
            ((pixel[2] as u16 * alpha + 255 * (255 - alpha) + 127) / 255) as u8,
        ])
    })
}

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    NotFound(String),
    Unauthorized(String),
    InternalServerError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response<Body> {
        let (status, error_type, message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request", msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "unauthorized", msg),
            AppError::InternalServerError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", msg)
            }
        };

        let body = serde_json::to_vec(&ErrorResponse {
            error: error_type.to_string(),
            message,
        })
        .unwrap_or_else(|_| b"Internal Server Error".to_vec());

        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );
        if status == StatusCode::UNAUTHORIZED {
            headers.insert(
                axum::http::header::WWW_AUTHENTICATE,
                "Bearer".parse().unwrap(),
            );
        }

        let mut res = Response::new(Body::from(body));
        *res.headers_mut() = headers;
        *res.status_mut() = status;
        res
    }
}

#[cfg(test)]
mod tests {
    use super::{authorize, constant_time_eq};
    use axum::http::{HeaderMap, HeaderValue};

    #[test]
    fn compares_api_keys() {
        assert!(constant_time_eq(b"secret", b"secret"));
        assert!(!constant_time_eq(b"secret", b"wrong"));
        assert!(!constant_time_eq(b"short", b"longer"));
    }

    #[test]
    fn accepts_header_and_bearer_api_keys() {
        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", HeaderValue::from_static("secret"));
        assert!(authorize(Some("secret"), &headers).is_ok());

        headers.remove("x-api-key");
        headers.insert("authorization", HeaderValue::from_static("Bearer secret"));
        assert!(authorize(Some("secret"), &headers).is_ok());
        assert!(authorize(Some("wrong"), &headers).is_err());
        assert!(authorize(None, &HeaderMap::new()).is_ok());
    }
}
