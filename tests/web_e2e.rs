#![cfg(feature = "server")]

use rembg_rs::server::{routes::create_router, state::AppState};
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use thirtyfour::prelude::*;

#[tokio::test]
#[ignore = "requires an ONNX model and ChromeDriver"]
async fn upload_process_and_download_result_and_mask() -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    let model_path = std::env::var("REMBG_E2E_MODEL")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("models/u2net.onnx"));
    let state = AppState::new(model_path, "127.0.0.1".to_string(), port)
        .await?
        .with_api_key(Some("e2e-secret".to_string()));
    tokio::spawn(async move {
        axum::serve(listener, create_router(state)).await.unwrap();
    });

    let mut capabilities = DesiredCapabilities::chrome();
    if let Ok(binary) = std::env::var("CHROME_BINARY") {
        capabilities.set_binary(&binary)?;
    }
    capabilities.add_arg("--headless")?;
    capabilities.add_arg("--window-size=1400,1000")?;
    capabilities.add_arg("--no-sandbox")?;
    capabilities.add_arg("--disable-dev-shm-usage")?;

    let download_dir = std::env::temp_dir().join(format!("rembg-rs-e2e-{}", std::process::id()));
    if download_dir.exists() {
        std::fs::remove_dir_all(&download_dir)?;
    }
    std::fs::create_dir_all(&download_dir)?;
    capabilities.add_experimental_option(
        "prefs",
        serde_json::json!({
            "download.default_directory": download_dir,
            "download.prompt_for_download": false,
            "download.directory_upgrade": true
        }),
    )?;

    let webdriver_url =
        std::env::var("WEBDRIVER_URL").unwrap_or_else(|_| "http://127.0.0.1:9515".to_string());
    let driver = WebDriver::builder(&webdriver_url, capabilities)
        .request_timeout(Duration::from_secs(60))
        .await?;

    let result = run_browser_flow(&driver, port, &download_dir).await;
    let quit_result = driver.quit().await;
    result.and(quit_result.map_err(Into::into))
}

async fn run_browser_flow(
    driver: &WebDriver,
    port: u16,
    download_dir: &Path,
) -> anyhow::Result<()> {
    driver.goto(format!("http://127.0.0.1:{port}")).await?;

    let api_key = driver
        .query(By::Css("[data-testid='api-key-input']"))
        .wait(Duration::from_secs(20), Duration::from_millis(100))
        .first()
        .await?;
    api_key.send_keys("e2e-secret").await?;
    driver
        .find(By::Css("[data-testid='api-key-submit']"))
        .await?
        .click()
        .await?;

    let upload = driver
        .query(By::Css("[data-testid='image-upload']"))
        .wait(Duration::from_secs(20), Duration::from_millis(100))
        .first()
        .await?;
    let image_path = std::fs::canonicalize("test-images/dragon.jpg")?;
    upload
        .send_keys(image_path.to_string_lossy().as_ref())
        .await?;

    driver
        .query(By::Css("[data-testid='include-mask']"))
        .wait(Duration::from_secs(10), Duration::from_millis(100))
        .first()
        .await?
        .click()
        .await?;
    driver
        .find(By::Css("[data-testid='process-image']"))
        .await?
        .click()
        .await?;

    let result_image = driver
        .query(By::Css("[data-testid='result-image']"))
        .wait(Duration::from_secs(180), Duration::from_millis(250))
        .first()
        .await?;
    let mask_image = driver
        .query(By::Css("[data-testid='mask-image']"))
        .wait(Duration::from_secs(20), Duration::from_millis(100))
        .first()
        .await?;

    for image in [result_image, mask_image] {
        let natural_width = image
            .prop("naturalWidth")
            .await?
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(0);
        assert!(natural_width > 0, "output preview did not decode");
        assert!(
            image
                .attr("src")
                .await?
                .is_some_and(|src| src.starts_with("blob:"))
        );
    }

    driver
        .find(By::Css("[data-testid='download-result']"))
        .await?
        .click()
        .await?;
    driver
        .find(By::Css("[data-testid='download-mask']"))
        .await?
        .click()
        .await?;

    for filename in ["dragon-removed.png", "dragon-mask.png"] {
        let path = download_dir.join(filename);
        wait_for_download(&path).await?;
        assert!(
            std::fs::metadata(&path)?.len() > 0,
            "download is empty: {filename}"
        );
        image::open(&path)?;
    }

    assert!(
        driver
            .find(By::Css("[data-testid='process-image']"))
            .await?
            .is_displayed()
            .await?
    );
    Ok(())
}

async fn wait_for_download(path: &Path) -> anyhow::Result<()> {
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(15) {
        if path.is_file() {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    anyhow::bail!("download did not appear: {}", path.display())
}
