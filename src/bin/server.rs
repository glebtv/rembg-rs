use clap::Parser;
use rembg_rs::server::routes::create_router;
use rembg_rs::server::state::AppState;
use std::io::BufRead;
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "rembg-server", about = "rembg-rs HTTP server")]
struct Args {
    #[arg(short, long, default_value = "u2net.onnx")]
    model: PathBuf,

    #[arg(short, long, default_value = "0.0.0.0")]
    host: String,

    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    #[arg(long)]
    no_build: bool,

    /// API key required by processing endpoints. REMBG_API_KEY is used when omitted.
    #[arg(long)]
    api_key: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("rembg_rs=info".parse()?)
                .add_directive("axum=info".parse()?),
        )
        .init();

    let args = Args::parse();

    tracing::info!("rembg-rs server starting...");

    if !args.no_build {
        tracing::info!("Building web UI...");
        if let Err(e) = run_web_build() {
            tracing::warn!("Web UI build skipped: {}", e);
        }
    } else {
        tracing::info!("Web UI build skipped (--no-build)");
    }

    let api_key = args.api_key.or_else(|| std::env::var("REMBG_API_KEY").ok());
    let app_state = AppState::new(args.model.clone(), args.host.clone(), args.port)
        .await?
        .with_api_key(api_key);

    if app_state.config.auth_required {
        tracing::info!("API key authentication enabled");
    } else {
        tracing::warn!("API key authentication disabled");
    }

    let app = create_router(app_state);
    let addr = SocketAddr::new(
        args.host
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid host"))?,
        args.port,
    );

    tracing::info!("Starting HTTP server on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn run_web_build() -> anyhow::Result<()> {
    if !std::path::Path::new("web/package.json").exists() {
        tracing::warn!("Web UI source not found (web/package.json missing)");
        return Ok(());
    }

    let start = std::time::Instant::now();
    tracing::info!("Running `npm run build` in web/");

    let mut child = std::process::Command::new("npm")
        .current_dir("web")
        .arg("run")
        .arg("build")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to run npm build: {}", e))?;

    // Stream output with progress indicator
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let reader = std::io::BufReader::new(stdout);
    let reader_err = std::io::BufReader::new(stderr);

    let progress_handle = std::thread::spawn(move || {
        let mut last_output = std::time::Instant::now();
        for line in reader.lines().map_while(Result::ok) {
            tracing::info!("  {}", line);
            last_output = std::time::Instant::now();
        }
        last_output
    });

    let _ = std::thread::spawn(move || {
        for line in reader_err.lines().map_while(Result::ok) {
            tracing::warn!("  {}", line);
        }
    })
    .join();

    let _last_output = progress_handle
        .join()
        .unwrap_or_else(|_| std::time::Instant::now());
    let status = child.wait()?;

    let elapsed = start.elapsed();
    if status.success() {
        tracing::info!("Web UI built successfully in {:.1}s", elapsed.as_secs_f32());
    } else {
        tracing::warn!(
            "Web UI build failed after {:.1}s: exit code {}",
            elapsed.as_secs_f32(),
            status.code().unwrap_or(-1)
        );
    }

    Ok(())
}
