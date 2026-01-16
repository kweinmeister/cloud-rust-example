use std::process::Command;
use std::io::Write;
use tempfile::NamedTempFile;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_main_starts_and_serves_requests() {
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
    let creds_path = creds_file.path().to_owned();

    let port = {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().port()
    };

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_cloud-rust"));
    let mut child = cmd
        .env("PORT", port.to_string())
        .env("GOOGLE_CLOUD_PROJECT", "test-project")
        .env("GOOGLE_APPLICATION_CREDENTIALS", creds_path)
        .spawn()
        .expect("Failed to spawn cloud-rust binary");

    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}", port);
    let mut ready = false;

    for _ in 0..30 {
        if let Ok(resp) = client.get(&url).send().await {
            if resp.status().is_success() {
                ready = true;
                
                let text = resp.text().await.unwrap();
                assert!(text.contains("Hello, World!"));
                break;
            }
        }
        sleep(Duration::from_millis(100)).await;
    }

    child.kill().expect("Failed to kill child process");
    
    assert!(ready, "Server never became ready or returned success");
}
