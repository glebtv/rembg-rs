use crate::manager::ModelManager;
use crate::models::models::ALL_MODELS;
use crate::server::types::ServerConfig;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone)]
pub struct AppState {
    pub model: Arc<tokio::sync::Mutex<ActiveModel>>,
    pub available_models: Arc<Vec<AvailableModel>>,
    pub model_path: PathBuf,
    pub config: ServerConfig,
    pub api_key: Option<Arc<str>>,
}

pub struct ActiveModel {
    pub manager: Option<ModelManager>,
    pub id: String,
}

#[derive(Clone)]
pub struct AvailableModel {
    pub id: String,
    pub name: String,
    pub description: String,
    pub path: PathBuf,
}

impl AppState {
    pub async fn new(
        model_path: PathBuf,
        host: String,
        port: u16,
    ) -> Result<Self, crate::error::RembgError> {
        let start = Instant::now();
        tracing::info!("Loading model: {:?}", model_path);

        let model = tokio::task::spawn_blocking({
            let path = model_path.clone();
            move || {
                let start = Instant::now();
                tracing::info!("  Reading model file...");
                let model = ModelManager::from_file(&path)?;
                let elapsed = start.elapsed();
                tracing::info!("  Model loaded in {:.1}s", elapsed.as_secs_f32());
                Ok::<_, crate::error::RembgError>(model)
            }
        })
        .await
        .map_err(|e| {
            crate::error::RembgError::InvalidInput(format!("Model loading task failed: {:?}", e))
        })??;

        let default_model = model_id(&model_path);
        let available_models = discover_models(&model_path);
        let elapsed = start.elapsed();
        tracing::info!("Server state initialized in {:.1}s", elapsed.as_secs_f32());

        Ok(Self {
            model: Arc::new(tokio::sync::Mutex::new(ActiveModel {
                manager: Some(model),
                id: default_model,
            })),
            available_models: Arc::new(available_models),
            config: ServerConfig {
                model_path: model_path.clone(),
                host,
                port,
                auth_required: false,
            },
            api_key: None,
            model_path,
        })
    }

    pub fn with_api_key(mut self, api_key: Option<String>) -> Self {
        self.api_key = api_key.filter(|key| !key.is_empty()).map(Arc::<str>::from);
        self.config.auth_required = self.api_key.is_some();
        self
    }
}

fn discover_models(default_path: &Path) -> Vec<AvailableModel> {
    let directory = default_path.parent().unwrap_or_else(|| Path::new("."));
    let mut paths = std::fs::read_dir(directory)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("onnx"))
        })
        .collect::<Vec<_>>();

    if !paths.iter().any(|path| path == default_path) {
        paths.push(default_path.to_path_buf());
    }
    paths.sort_by_key(|path| model_id(path));
    paths.dedup_by(|left, right| model_id(left) == model_id(right));

    paths
        .into_iter()
        .map(|path| {
            let id = model_id(&path);
            let definition = ALL_MODELS.iter().find(|model| model.filename == id);
            AvailableModel {
                name: definition.map_or_else(
                    || model_name(&id),
                    |model| {
                        model
                            .description
                            .split(" - ")
                            .next()
                            .unwrap_or(model.name)
                            .to_string()
                    },
                ),
                description: definition
                    .map(|model| model.description.to_string())
                    .unwrap_or_else(|| "Custom ONNX model".to_string()),
                id,
                path,
            }
        })
        .collect()
}

fn model_id(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("model.onnx")
        .to_string()
}

fn model_name(id: &str) -> String {
    id.strip_suffix(".onnx")
        .unwrap_or(id)
        .replace(['-', '_'], " ")
}
