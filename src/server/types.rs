use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerConfig {
    pub model_path: PathBuf,
    pub host: String,
    pub port: u16,
    pub auth_required: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveBgRequest {
    pub threshold: Option<u8>,
    pub binary: Option<bool>,
    pub sticker: Option<bool>,
    pub compress: Option<bool>,
    pub format: Option<String>,
    pub include_mask: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveBgResponse {
    pub image_data: String,
    pub image_format: String,
    pub filename: String,
    pub mask_data: Option<String>,
    pub mask_format: Option<String>,
    pub mask_filename: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigResponse {
    pub model_path: String,
    pub host: String,
    pub port: u16,
    pub auth_required: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}
