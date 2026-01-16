use axum::{Router, extract::Extension, response::Html, routing::get};
use google_cloud_resourcemanager_v3::client::Projects;
use std::{
    env::var,
    sync::{Arc, OnceLock},
};
use tokio::net::TcpListener;

static PROJECT_ID: OnceLock<String> = OnceLock::new();

#[tokio::main]
async fn main() {
    // create a the Resource Manager Projects client
    let client = Projects::builder().build().await.unwrap();

    // get the project ID
    let project_id = get_project_id().await.expect("Failed to get project ID");
    PROJECT_ID
        .set(project_id)
        .expect("Failed to set PROJECT_ID");

    // get the port from the environment, defaulting to 8080
    let port = var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);

    // build our application with routes
    let app = Router::new()
        .route("/", get(handler))
        .route("/project", get(project_handler))
        .layer(Extension(Arc::new(client)));

    // run it
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}

async fn project_handler(Extension(client): Extension<Arc<Projects>>) -> Html<String> {
    let project_id = PROJECT_ID.get().expect("Project ID not initialized");
    let project_name = format!("projects/{}", project_id);

    match client.get_project().set_name(project_name).send().await {
        Ok(project) => {
            let project_number = project.name.strip_prefix("projects/").unwrap_or("Unknown");

            Html(format!(
                "<h1>Project Info</h1><ul><li>Name: <code>{}</code></li><li>ID: <code>{}</code></li><li>Number: <code>{}</code></li></ul>",
                &project.display_name, project_id, project_number
            ))
        }
        Err(e) => Html(format!("<h1>Error getting project info: {}</h1>", e)),
    }
}

// Helper function to get the project ID.
async fn get_project_id() -> Result<String, String> {
    if let Ok(project_id) = var("GOOGLE_CLOUD_PROJECT") {
        return Ok(project_id);
    }

    let client = reqwest::Client::new();
    let url = "http://metadata.google.internal/computeMetadata/v1/project/project-id";

    let response = client
        .get(url)
        .header("Metadata-Flavor", "Google")
        .send()
        .await;

    match response {
        Ok(res) => {
            if res.status().is_success() {
                Ok(res.text().await.map_err(|e| e.to_string())?)
            } else {
                Err(format!("Metadata server returned error: {}", res.status()))
            }
        }
        Err(e) => Err(format!("Error querying metadata server: {}", e)),
    }
}
