//! Model definitions for downloading pre-trained models
//!
//! Models are sourced from the rembg project: https://github.com/danielgatis/rembg

/// Supported model names
pub const SUPPORTED_MODELS: &[&str] = &[
    "u2net",
    "u2netp",
    "u2net_human_seg",
    "u2net_cloth_seg",
    "isnet-general-use",
    "isnet-anime",
    "silueta",
    "bria-rmbg",
    "birefnet-general",
    "birefnet-general-lite",
    "birefnet-portrait",
    "birefnet-dis",
    "birefnet-hrsod",
    "birefnet-cod",
    "birefnet-massive",
];

/// All available models with their download URLs and checksums
pub const ALL_MODELS: &[ModelDef] = &[
    ModelDef {
        name: "u2net",
        filename: "u2net.onnx",
        url: "https://github.com/danielgatis/rembg/releases/download/v0.0.0/u2net.onnx",
        checksum_type: ChecksumType::Md5,
        checksum: "60024c5c889badc19c04ad937298a77b",
        description: "U2Net - Universal background removal (general purpose)",
    },
    ModelDef {
        name: "u2netp",
        filename: "u2netp.onnx",
        url: "https://github.com/danielgatis/rembg/releases/download/v0.0.0/u2netp.onnx",
        checksum_type: ChecksumType::Md5,
        checksum: "8e83ca70e441ab06c318d82300c84806",
        description: "U2NetP - Portable/lightweight variant of U2Net",
    },
    ModelDef {
        name: "u2net_human_seg",
        filename: "u2net_human_seg.onnx",
        url: "https://github.com/danielgatis/rembg/releases/download/v0.0.0/u2net_human_seg.onnx",
        checksum_type: ChecksumType::Md5,
        checksum: "c09ddc2e0104f800e3e1bb4652583d1f",
        description: "U2Net Human Segmentation - Specialized for human subjects",
    },
    ModelDef {
        name: "u2net_cloth_seg",
        filename: "u2net_cloth_seg.onnx",
        url: "https://github.com/danielgatis/rembg/releases/download/v0.0.0/u2net_cloth_seg.onnx",
        checksum_type: ChecksumType::Md5,
        checksum: "2434d1f3cb744e0e49386c906e5a08bb",
        description: "U2Net Cloth Segmentation - Clothing detection",
    },
    ModelDef {
        name: "isnet-general-use",
        filename: "isnet-general-use.onnx",
        url: "https://github.com/danielgatis/rembg/releases/download/v0.0.0/isnet-general-use.onnx",
        checksum_type: ChecksumType::Md5,
        checksum: "fc16ebd8b0c10d971d3513d564d01e29",
        description: "IS-Net General Use - General purpose matting network",
    },
    ModelDef {
        name: "isnet-anime",
        filename: "isnet-anime.onnx",
        url: "https://github.com/danielgatis/rembg/releases/download/v0.0.0/isnet-anime.onnx",
        checksum_type: ChecksumType::Md5,
        checksum: "6f184e756bb3bd901c8849220a83e38e",
        description: "IS-Net Anime - Specialized for anime/cartoon images",
    },
    ModelDef {
        name: "silueta",
        filename: "silueta.onnx",
        url: "https://github.com/danielgatis/rembg/releases/download/v0.0.0/silueta.onnx",
        checksum_type: ChecksumType::Md5,
        checksum: "55e59e0d8062d2f5d013f4725ee84782",
        description: "Silueta - Lightweight model for simple foreground extraction",
    },
    ModelDef {
        name: "bria-rmbg",
        filename: "bria-rmbg-2.0.onnx",
        url: "https://github.com/danielgatis/rembg/releases/download/v0.0.0/bria-rmbg-2.0.onnx",
        checksum_type: ChecksumType::Sha256,
        checksum: "5b486f08200f513f460da46dd701db5fbb47d79b4be4b708a19444bcd4e79958",
        description: "BRIA RMBG 2.0 - State-of-the-art background removal",
    },
    ModelDef {
        name: "birefnet-general",
        filename: "BiRefNet-general-epoch_244.onnx",
        url: "https://github.com/danielgatis/rembg/releases/download/v0.0.0/BiRefNet-general-epoch_244.onnx",
        checksum_type: ChecksumType::Md5,
        checksum: "7a35a0141cbbc80de11d9c9a28f52697",
        description: "BiRefNet General - High-quality general-purpose model",
    },
    ModelDef {
        name: "birefnet-general-lite",
        filename: "BiRefNet-general-bb_swin_v1_tiny-epoch_232.onnx",
        url: "https://github.com/danielgatis/rembg/releases/download/v0.0.0/BiRefNet-general-bb_swin_v1_tiny-epoch_232.onnx",
        checksum_type: ChecksumType::Md5,
        checksum: "4fab47adc4ff364be1713e97b7e66334",
        description: "BiRefNet General Lite - Lightweight variant",
    },
    ModelDef {
        name: "birefnet-portrait",
        filename: "BiRefNet-portrait-epoch_150.onnx",
        url: "https://github.com/danielgatis/rembg/releases/download/v0.0.0/BiRefNet-portrait-epoch_150.onnx",
        checksum_type: ChecksumType::Md5,
        checksum: "c3a64a6abf20250d090cd055f12a3b67",
        description: "BiRefNet Portrait - Optimized for portrait images",
    },
    ModelDef {
        name: "birefnet-dis",
        filename: "BiRefNet-DIS-epoch_590.onnx",
        url: "https://github.com/danielgatis/rembg/releases/download/v0.0.0/BiRefNet-DIS-epoch_590.onnx",
        checksum_type: ChecksumType::Md5,
        checksum: "2d4d44102b446f33a4ebb2e56c051f2b",
        description: "BiRefNet DIS - Distinct object separation",
    },
    ModelDef {
        name: "birefnet-hrsod",
        filename: "BiRefNet-HRSOD_DHU-epoch_115.onnx",
        url: "https://github.com/danielgatis/rembg/releases/download/v0.0.0/BiRefNet-HRSOD_DHU-epoch_115.onnx",
        checksum_type: ChecksumType::Md5,
        checksum: "c017ade5de8a50ff0fd74d790d268dda",
        description: "BiRefNet HRSOD - High-resolution salient object detection",
    },
    ModelDef {
        name: "birefnet-cod",
        filename: "BiRefNet-COD-epoch_125.onnx",
        url: "https://github.com/danielgatis/rembg/releases/download/v0.0.0/BiRefNet-COD-epoch_125.onnx",
        checksum_type: ChecksumType::Md5,
        checksum: "f6d0d21ca89d287f17e7afe9f5fd3b45",
        description: "BiRefNet COD - Camouflaged object detection",
    },
    ModelDef {
        name: "birefnet-massive",
        filename: "BiRefNet-massive-TR_DIS5K_TR_TEs-epoch_420.onnx",
        url: "https://github.com/danielgatis/rembg/releases/download/v0.0.0/BiRefNet-massive-TR_DIS5K_TR_TEs-epoch_420.onnx",
        checksum_type: ChecksumType::Md5,
        checksum: "33e726a2136a3d59eb0fdf613e31e3e9",
        description: "BiRefNet Massive - Large-scale trained model",
    },
];

/// Model definition
#[derive(Debug, Clone)]
pub struct ModelDef {
    pub name: &'static str,
    pub filename: &'static str,
    pub url: &'static str,
    pub checksum_type: ChecksumType,
    pub checksum: &'static str,
    pub description: &'static str,
}

/// Type of checksum used for verification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChecksumType {
    Md5,
    Sha256,
}

/// Result of a model download operation
#[derive(Debug)]
pub struct DownloadResult {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub duration_secs: f32,
    pub verified: bool,
}

impl ModelDef {
    /// Get model definition by name
    pub fn by_name(name: &str) -> Option<&'static Self> {
        ALL_MODELS.iter().find(|m| m.name == name)
    }

    /// Format size in human-readable form
    pub fn format_size(bytes: u64) -> String {
        if bytes >= 1_073_741_824 {
            format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
        } else if bytes >= 1_048_576 {
            format!("{:.1} MB", bytes as f64 / 1_048_576.0)
        } else if bytes >= 1_024 {
            format!("{:.1} KB", bytes as f64 / 1_024.0)
        } else {
            format!("{} B", bytes)
        }
    }
}
