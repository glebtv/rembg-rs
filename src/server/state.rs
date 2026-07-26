use crate::manager::ModelManager;
use crate::server::types::ServerConfig;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone)]
pub struct AppState {
    pub model: Arc<tokio::sync::Mutex<ModelManager>>,
    pub model_path: PathBuf,
    pub config: ServerConfig,
    pub api_key: Option<Arc<str>>,
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

        let elapsed = start.elapsed();
        tracing::info!("Server state initialized in {:.1}s", elapsed.as_secs_f32());

        Ok(Self {
            model: Arc::new(tokio::sync::Mutex::new(model)),
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
