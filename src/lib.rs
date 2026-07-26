mod clean_sticker_border;
#[cfg(feature = "cli")]
pub mod cli;
pub mod compress_png;
pub mod error;
pub mod manager;
pub mod models;
pub mod options;
pub mod rembg;
pub mod result;

#[cfg(feature = "server")]
pub mod server;
