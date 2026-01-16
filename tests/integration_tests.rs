use rstest::*;
use std::io::Write;
use std::process::Command;
use std::time::Duration;
use tempfile::NamedTempFile;
use tokio::time::sleep;

const MAX_RETRIES: usize = 30;
const RETRY_DELAY_MS: u64 = 100;

#[fixture]
fn creds_file() -> NamedTempFile {
    let mut creds_file = NamedTempFile::new().expect("Failed to create temp file");
    let creds_json = r#"{
        "type": "service_account",
        "project_id": "test-project",
        "private_key_id": "fake-key-id",
        "private_key": "fake-private-key",
        "client_email": "fake-email@example.com",
        "client_id": "fake-client-id",
        "auth_uri": "https://accounts.google.com/o/oauth2/auth",
        "token_uri": "https://oauth2.googleapis.com/token",
        "auth_provider_x509_cert_url": "https://www.googleapis.com/oauth2/v1/certs",
        "client_x509_cert_url": "https://www.googleapis.com/robot/v1/metadata/x509/fake-email%40example.com",
        "universe_domain": "googleapis.com"
    }"#;
    write!(creds_file, "{}", creds_json).expect("Failed to write to temp creds file");
    creds_file
}

#[fixture]
fn port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

async fn wait_for_server_ready(port: u16) {
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}", port);
    let mut ready = false;

    for _ in 0..MAX_RETRIES {
        if let Ok(resp) = client.get(&url).send().await {
            if resp.status().is_success() {
                ready = true;
                break;
            }
        }
        sleep(Duration::from_millis(RETRY_DELAY_MS)).await;
    }

    assert!(ready, "Server never became ready");
}

#[rstest]
#[tokio::test]
async fn test_main_starts_and_serves_requests(creds_file: NamedTempFile, port: u16) {
    let creds_path = creds_file.path().to_owned();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_cloud-rust"));
    let mut child = cmd
        .env("PORT", port.to_string())
        .env("GOOGLE_CLOUD_PROJECT", "test-project")
        .env("GOOGLE_APPLICATION_CREDENTIALS", creds_path)
        .spawn()
        .expect("Failed to spawn cloud-rust binary");

    wait_for_server_ready(port).await;

    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}", port);
    let resp = client
        .get(&url)
        .send()
        .await
        .expect("Failed to request root");
    let text = resp.text().await.unwrap();
    assert!(text.contains("Hello, World!"));

    child.kill().expect("Failed to kill child process");
}

#[rstest]
#[tokio::test]
async fn test_project_endpoint_handles_network_failure(creds_file: NamedTempFile, port: u16) {
    let creds_path = creds_file.path().to_owned();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_cloud-rust"));
    let mut child = cmd
        .env("PORT", port.to_string())
        .env("GOOGLE_CLOUD_PROJECT", "test-project")
        .env("GOOGLE_APPLICATION_CREDENTIALS", creds_path)
        .env("HTTPS_PROXY", "http://0.0.0.0:0") // Force connection failure locally
        .spawn()
        .expect("Failed to spawn cloud-rust binary");

    wait_for_server_ready(port).await;

    let client = reqwest::Client::new();
    let project_url = format!("http://127.0.0.1:{}/project", port);

    let resp = client
        .get(&project_url)
        .send()
        .await
        .expect("Failed to send request to /project");

    assert!(resp.status().is_success());
    let text = resp.text().await.unwrap();
    assert!(
        text.contains("Error getting project info"),
        "Response did not contain expected error message. Got: {}",
        text
    );

    child.kill().expect("Failed to kill child process");
}
