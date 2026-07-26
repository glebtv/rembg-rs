use crate::error::RembgError;
use ndarray::Array;
use ort::{
    ep,
    session::{
        Session,
        builder::{GraphOptimizationLevel, SessionBuilder},
    },
    value::Value,
    value::ValueType,
};
use std::path::{Path, PathBuf};

pub struct ModelManager {
    session: Session,
    input_width: u32,
    input_height: u32,
}

impl ModelManager {
    /// Create a new model manager from model file
    ///
    /// Uses memory mapping - OS decides whether to keep model in RAM or load on demand.
    /// This is the most memory-efficient approach for long-running applications.
    pub fn from_file(model_path: &Path) -> Result<Self, RembgError> {
        if !model_path.is_file() {
            return Err(RembgError::ModelNotFound(model_path.display().to_string()));
        }
        ensure_onnxruntime_available()?;

        // Create session with model file (uses memory mapping)
        let mut session_builder = SessionBuilder::new().map_err(RembgError::OnnxError)?;
        session_builder = session_builder
            .with_execution_providers([ep::CUDA::default().build(), ep::CPU::default().build()])
            .map_err(|e| RembgError::OnnxError(e.into()))?;
        session_builder = session_builder
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| RembgError::OnnxError(e.into()))?;
        session_builder = session_builder
            .with_intra_threads(4)
            .map_err(|e| RembgError::OnnxError(e.into()))?;
        let session = session_builder
            .commit_from_file(model_path)
            .map_err(RembgError::OnnxError)?;

        let (input_width, input_height) = match session.inputs().first().map(|input| input.dtype())
        {
            Some(ValueType::Tensor { shape, .. }) if shape.len() == 4 => {
                let height = shape[2];
                let width = shape[3];
                if width > 0 && height > 0 {
                    (width as u32, height as u32)
                } else {
                    dynamic_input_dimensions(model_path)
                }
            }
            _ => dynamic_input_dimensions(model_path),
        };

        Ok(Self {
            session,
            input_width,
            input_height,
        })
    }

    pub fn input_dimensions(&self) -> (u32, u32) {
        (self.input_width, self.input_height)
    }

    /// Run inference on preprocessed input
    pub fn run_inference(
        &mut self,
        input: &ndarray::Array4<f32>,
    ) -> Result<ndarray::Array4<f32>, RembgError> {
        // Convert to tuple format (shape, data) which ort accepts
        let shape = input.shape().to_vec();
        let data: Vec<f32> = input.iter().copied().collect();

        // Create input tensor from tuple (shape, data)
        let input_tensor = Value::from_array((shape.as_slice(), data))?;

        // Get input/output names before running inference (to avoid borrow issues)
        let input_name = self.session.inputs()[0].name().to_string();
        let output_name = self.session.outputs()[0].name().to_string();

        // Run inference
        let outputs = self
            .session
            .run(ort::inputs![input_name.as_str() => input_tensor])?;

        // Extract output tensor by name
        let output = outputs
            .get(output_name.as_str())
            .ok_or_else(|| RembgError::TensorError("No output from model".to_string()))?;

        // Extract tensor data - try_extract_tensor returns (shape, data slice)
        let (shape, data) = output
            .try_extract_tensor::<f32>()
            .map_err(|e| RembgError::TensorError(format!("Failed to extract tensor: {}", e)))?;

        // Convert shape from i64 to usize
        let shape_vec: Vec<usize> = shape.as_ref().iter().map(|&x| x as usize).collect();

        // Create ndarray from data
        let output_array =
            Array::from_shape_vec(shape_vec.as_slice(), data.to_vec()).map_err(|e| {
                RembgError::TensorError(format!("Failed to create output array: {}", e))
            })?;

        // Reshape to 4D if needed
        let output_shape = output_array.shape();
        let output_4d = if output_shape.len() == 4 {
            output_array.into_dimensionality()?
        } else if output_shape.len() == 3 {
            output_array
                .insert_axis(ndarray::Axis(0))
                .into_dimensionality()?
        } else if output_shape.len() == 2 {
            // Add batch and channel dimensions
            output_array
                .insert_axis(ndarray::Axis(0))
                .insert_axis(ndarray::Axis(0))
                .into_dimensionality()?
        } else {
            return Err(RembgError::TensorError(format!(
                "Unexpected output shape: {:?}",
                output_shape
            )));
        };

        Ok(output_4d)
    }
}

fn ensure_onnxruntime_available() -> Result<(), RembgError> {
    let library = std::env::var_os("ORT_DYLIB_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(onnxruntime_library_name()));

    // SAFETY: The handle is only used to verify that the runtime can be loaded. `ort`
    // opens and owns its own handle immediately afterward.
    unsafe { libloading::Library::new(&library) }
        .map(|_| ())
        .map_err(|error| {
            RembgError::OnnxRuntimeNotAvailable(format!(
                "could not load {}: {}. Install ONNX Runtime or set ORT_DYLIB_PATH",
                library.display(),
                error
            ))
        })
}

fn dynamic_input_dimensions(model_path: &Path) -> (u32, u32) {
    let filename = model_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if filename.contains("bria-rmbg") {
        (1024, 1024)
    } else {
        (320, 320)
    }
}

#[cfg(target_os = "windows")]
fn onnxruntime_library_name() -> &'static str {
    "onnxruntime.dll"
}

#[cfg(target_os = "macos")]
fn onnxruntime_library_name() -> &'static str {
    "libonnxruntime.dylib"
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
fn onnxruntime_library_name() -> &'static str {
    "libonnxruntime.so"
}
