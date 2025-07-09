use axum::{routing::{get, put}, Router, extract::{Path, Json}, response::{IntoResponse, Response}, http::StatusCode, body::Body};
use hyper::Server;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tower_http::cors::CorsLayer;
use std::fs;
use std::path::Path as FsPath;

#[derive(Serialize, Deserialize, Clone)]
struct PackageMeta {
    name: String,
    versions: HashMap<String, String>, // version -> description
}

type Registry = Arc<Mutex<HashMap<String, PackageMeta>>>;

#[tokio::main]
async fn main() {
    let registry: Registry = Arc::new(Mutex::new(HashMap::new()));
    let app = Router::new()
        .route("/v1/:pkg/", put(publish_package).get(get_metadata))
        .route("/v1/:pkg/:version/", get(get_tarball))
        .route("/v1/index", get(get_index))
        .layer(CorsLayer::permissive());
    println!("BoltPM Registry running on http://localhost:4000");
    Server::bind(&"0.0.0.0:4000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn publish_package(Path(pkg): Path<String>, Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    // TODO: Check token (stub)
    // Save .tgz and update packages.json
    let version = body["version"].as_str().unwrap_or("0.1.0");
    let desc = body["description"].as_str().unwrap_or("");
    let pkg_dir = format!("packages/{}/{}", pkg, version);
    fs::create_dir_all(&pkg_dir).ok();
    // Simulate saving .tgz (stub)
    fs::write(format!("{}/package.tgz", pkg_dir), b"stub-tgz").ok();
    // Update packages.json
    let meta_path = FsPath::new("packages/packages.json");
    let mut meta: HashMap<String, PackageMeta> = if meta_path.exists() {
        serde_json::from_slice(&fs::read(meta_path).unwrap()).unwrap()
    } else {
        HashMap::new()
    };
    let entry = meta.entry(pkg.clone()).or_insert(PackageMeta {
        name: pkg.clone(),
        versions: HashMap::new(),
    });
    entry.versions.insert(version.to_string(), desc.to_string());
    fs::create_dir_all("packages").ok();
    fs::write(meta_path, serde_json::to_vec_pretty(&meta).unwrap()).ok();
    (StatusCode::OK, format!("Published package: {}@{} (stub)", pkg, version))
}

async fn get_metadata(Path(pkg): Path<String>) -> impl IntoResponse {
    let meta_path = FsPath::new("packages/packages.json");
    if !meta_path.exists() {
        return (StatusCode::NOT_FOUND, "No metadata".to_string());
    }
    let meta: HashMap<String, PackageMeta> = serde_json::from_slice(&fs::read(meta_path).unwrap()).unwrap();
    if let Some(pkg_meta) = meta.get(&pkg) {
        (StatusCode::OK, serde_json::to_string_pretty(pkg_meta).unwrap())
    } else {
        (StatusCode::NOT_FOUND, "Package not found".to_string())
    }
}

async fn get_tarball(Path((pkg, version)): Path<(String, String)>) -> Response {
    let path = format!("packages/{}/{}/package.tgz", pkg, version);
    if let Ok(bytes) = fs::read(&path) {
        Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(bytes))
            .unwrap()
    } else {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Tarball not found"))
            .unwrap()
    }
}

async fn get_index() -> impl IntoResponse {
    let meta_path = FsPath::new("packages/packages.json");
    if !meta_path.exists() {
        return (StatusCode::OK, "{}".to_string());
    }
    let meta: HashMap<String, PackageMeta> = serde_json::from_slice(&fs::read(meta_path).unwrap()).unwrap();
    (StatusCode::OK, serde_json::to_string_pretty(&meta).unwrap())
} 