# cloud-rust-example

This is an example web application built using the [axum](https://github.com/tokio-rs/axum) framework in Rust, designed for deployment on Google Cloud Run. The application was derived from the axum [hello-world example](https://github.com/tokio-rs/axum/blob/main/examples/hello-world).

## Description

The application exposes two routes:

- `/`: Returns an HTML response with the text "Hello, world!".
- `/project`: Returns information about the Google Cloud project the application is running in.

It's designed to be a minimal example of a web server in Rust, demonstrating integration with Google Cloud services. The server listens on port 8080 by default, but this can be configured using the `PORT` environment variable.

## Dependencies

The project has the following dependencies:

- `axum`: Web application framework
- `tokio`: Asynchronous runtime
- `google-cloud-resourcemanager-v3`: Google Cloud Resource Manager API client
- `reqwest`: HTTP client

These dependencies are managed by Cargo, Rust's package manager and build system.

## Running Locally

To run the application locally, you'll need Rust and Cargo installed. You can then use the following command:

```bash
cargo run
```

This will compile and start the web server. You can access the application by navigating to `http://localhost:8080` (or the port specified by the `PORT` environment variable) in your web browser.

For local testing, you'll also need to set the `GOOGLE_CLOUD_PROJECT` environment variable to your Google Cloud project ID:

```bash
export GOOGLE_CLOUD_PROJECT=your-project-id
```

Replace `your-project-id` with your actual project ID.

## Deploying to Cloud Run

To deploy the application to Cloud Run, use the following command:

```bash
gcloud run deploy cloud-rust-example \
    --source . \
    --region us-central1 \
    --allow-unauthenticated
```

This command will build and deploy your application to Cloud Run. The `--allow-unauthenticated` flag makes the service publicly accessible for testing. For production applications, you should remove this flag and implement proper authentication.
