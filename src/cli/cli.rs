use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "rembg-rs",
    about = "A Rust utility for removing backgrounds from art-style images using neural networks",
    version = env!("CARGO_PKG_VERSION"),
    author = "WarRaft"
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Remove background from an image
    #[command(name = "remove-bg", about = "Remove background from an image")]
    RemoveBg(RemoveBgArgs),

    /// Download and cache pre-trained models
    #[cfg(feature = "download")]
    #[command(
        name = "download-models",
        about = "Download and cache pre-trained models"
    )]
    DownloadModels(DownloadModelsArgs),
}

#[derive(Parser)]
pub struct RemoveBgArgs {
    /// Input image file path
    #[arg(short = 'i', long = "input", help = "Path to the input image file")]
    pub input: PathBuf,

    /// Output image file path
    #[arg(
        short = 'o',
        long = "output",
        help = "Path where the output image will be saved"
    )]
    pub output: PathBuf,

    /// Model file path (ONNX format)
    #[arg(
        short = 'm',
        long = "model",
        default_value = "u2net.onnx",
        help = "Path to the ONNX model file"
    )]
    pub model: String,

    /// Alpha matting threshold (0-255)
    #[arg(
        short = 't',
        long = "threshold",
        default_value = "160",
        help = "Threshold for alpha matting (0-255). Higher values = more aggressive removal"
    )]
    pub threshold: u8,

    /// Enable binary mask mode (no transparency, just cut out)
    #[arg(
        short = 'b',
        long = "binary",
        help = "Use binary mask (full transparent or opaque, no semi-transparency)"
    )]
    pub binary: bool,

    /// Save mask as separate file
    #[arg(
        short = 's',
        long = "save-mask",
        help = "Save the mask as a separate grayscale image alongside the output"
    )]
    pub save_mask: bool,

    /// Process as sticker (add outline)
    #[arg(long = "sticker", help = "Process sticker")]
    pub sticker: bool,

    /// Skip optional PNG quantization
    #[arg(
        long = "no-compress",
        help = "Save the output without PNG quantization"
    )]
    pub no_compress: bool,
}

#[derive(Parser)]
#[cfg(feature = "download")]
pub struct DownloadModelsArgs {
    /// Download only these models (comma-separated, or 'all' for everything)
    #[arg(
        short = 'n',
        long = "name",
        default_value = "all",
        help = "Model names to download (comma-separated). Use 'all' for everything. Available: u2net, u2netp, u2net_human_seg, u2net_cloth_seg, isnet-general-use, isnet-anime, silueta, bria-rmbg, birefnet-general, birefnet-general-lite, birefnet-portrait, birefnet-dis, birefnet-hrsod, birefnet-cod, birefnet-massive"
    )]
    pub names: String,

    /// Directory to store models (default: ./models)
    #[arg(
        short = 'd',
        long = "dir",
        default_value = "models",
        help = "Directory to store downloaded models"
    )]
    pub dir: PathBuf,

    /// Skip checksum verification
    #[arg(
        long = "skip-verify",
        help = "Skip checksum verification after download"
    )]
    pub skip_verify: bool,

    /// Force re-download even if file exists
    #[arg(
        long = "force",
        help = "Force re-download even if file exists and verified"
    )]
    pub force: bool,
}
