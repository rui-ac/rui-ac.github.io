use axum::{
    body::Body,
    extract::Json,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use serde::Deserialize;
use std::{io::Write, path::Path, process::Stdio};
use tokio::process::Command;
use tower_http::cors::{Any, CorsLayer};
use walkdir::WalkDir;

#[derive(Deserialize)]
struct DownloadRequest {
    url: String,
}

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/download", post(download))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on port 3000");
    axum::serve(listener, app).await.unwrap();
}

async fn download(Json(payload): Json<DownloadRequest>) -> Response {
    let url = payload.url.trim().to_string();
    if url.is_empty() {
        return (StatusCode::BAD_REQUEST, "URL is required").into_response();
    }

    let temp_dir = match tempfile::tempdir() {
        Ok(d) => d,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create temp dir: {e}"),
            )
                .into_response()
        }
    };

    let result = Command::new("gallery-dl")
        .args(["-d", temp_dir.path().to_str().unwrap(), "--no-mtime", &url])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await;

    match result {
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to run gallery-dl: {e}"),
            )
                .into_response()
        }
        Ok(out) if !out.status.success() => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("gallery-dl error: {stderr}"),
            )
                .into_response();
        }
        Ok(_) => {}
    }

    // Collect all downloaded files (non-directories)
    let files: Vec<_> = WalkDir::new(temp_dir.path())
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
        .collect();

    if files.is_empty() {
        return (StatusCode::NOT_FOUND, "No media found at that URL").into_response();
    }

    if files.len() == 1 {
        send_single_file(&files[0]).await
    } else {
        send_zip(&files, temp_dir.path()).await
    }
}

async fn send_single_file(path: &Path) -> Response {
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("download")
        .to_string();

    let content_type = content_type_for(path);

    match tokio::fs::read(path).await {
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to read file: {e}"),
        )
            .into_response(),
        Ok(bytes) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, content_type)
            .header(
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}\""),
            )
            .body(Body::from(bytes))
            .unwrap(),
    }
}

async fn send_zip(files: &[std::path::PathBuf], base: &Path) -> Response {
    let buf: Vec<u8> = Vec::new();
    let cursor = std::io::Cursor::new(buf);

    let result = tokio::task::spawn_blocking({
        let files = files.to_vec();
        let base = base.to_path_buf();
        move || -> std::io::Result<Vec<u8>> {
            let mut zip = zip::ZipWriter::new(cursor);
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);

            for path in &files {
                let name = path
                    .strip_prefix(&base)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .to_string();
                zip.start_file(&name, options)?;
                let data = std::fs::read(path)?;
                zip.write_all(&data)?;
            }

            let result = zip.finish()?;
            Ok(result.into_inner())
        }
    })
    .await;

    match result {
        Ok(Ok(bytes)) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/zip")
            .header(
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"download.zip\"",
            )
            .body(Body::from(bytes))
            .unwrap(),
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create zip: {e}"),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Task failed: {e}"),
        )
            .into_response(),
    }
}

fn content_type_for(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .as_deref()
    {
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        Some("mov") => "video/quicktime",
        Some("mkv") => "video/x-matroska",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        _ => "application/octet-stream",
    }
}
