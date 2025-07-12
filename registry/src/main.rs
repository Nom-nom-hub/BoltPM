use axum::{
    body::Body,
    extract::{Json, Multipart, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::net::SocketAddr;
use std::path::Path as FsPath;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

#[derive(Serialize, Deserialize, Clone, Default)]
struct VersionMeta {
    description: String,
    yanked: bool,
    deprecated: bool,
    deprecation_message: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct PackageMeta {
    name: String,
    versions: HashMap<String, VersionMeta>, // version -> VersionMeta
}

type Registry = Arc<Mutex<HashMap<String, PackageMeta>>>;

fn migrate_meta(meta_raw: HashMap<String, PackageMeta>) -> HashMap<String, PackageMeta> {
    use serde_json::Value;
    let mut new_meta = HashMap::new();
    for (pkg_name, pkg) in meta_raw {
        let mut new_versions = HashMap::new();
        // Deserialize versions as Value for migration
        let versions_val: HashMap<String, Value> =
            serde_json::from_value(serde_json::to_value(pkg.versions).unwrap()).unwrap();
        for (ver, val) in versions_val {
            if let Value::String(desc) = val {
                new_versions.insert(
                    ver,
                    VersionMeta {
                        description: desc,
                        yanked: false,
                        deprecated: false,
                        deprecation_message: None,
                    },
                );
            } else if let Ok(vm) = serde_json::from_value::<VersionMeta>(val) {
                new_versions.insert(ver, vm);
            }
        }
        new_meta.insert(
            pkg_name,
            PackageMeta {
                name: pkg.name,
                versions: new_versions,
            },
        );
    }
    new_meta
}

#[tokio::main]
async fn main() {
    let _registry: Registry = Arc::new(Mutex::new(HashMap::new()));
    let app = Router::new()
        .route("/v1/:pkg/", put(publish_package).get(get_metadata))
        .route("/v1/:pkg/:version/", get(get_tarball))
        .route("/v1/:pkg/:version/yank", post(yank_version))
        .route("/v1/:pkg/:version/unyank", post(unyank_version))
        .route("/v1/:pkg/:version/deprecate", post(deprecate_version))
        .route("/v1/index", get(get_index))
        .route("/v1/search", get(search_packages))
        .layer(CorsLayer::permissive());
    println!("BoltPM Registry running on http://localhost:4000");
    let addr = "0.0.0.0:4000".parse::<SocketAddr>().unwrap();
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn publish_package(
    Path(pkg): Path<String>,
    mut multipart: Multipart,
) -> (StatusCode, String) {
    // TODO: Check token (stub)
    let mut version = None;
    let mut desc = None;
    let mut tarball_bytes = None;
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap_or("");
        match name {
            "version" => version = Some(field.text().await.unwrap()),
            "description" => desc = Some(field.text().await.unwrap()),
            "tarball" => tarball_bytes = Some(field.bytes().await.unwrap()),
            _ => {}
        }
    }
    let version = version.unwrap_or("0.1.0".to_string());
    let desc = desc.unwrap_or_default();
    let tarball_bytes = match tarball_bytes {
        Some(b) => b,
        None => return (StatusCode::BAD_REQUEST, "Missing tarball".to_string()),
    };
    let pkg_dir = format!("packages/{pkg}/{version}");
    fs::create_dir_all(&pkg_dir).ok();
    fs::write(format!("{pkg_dir}/package.tgz"), &tarball_bytes).ok();
    // Update packages.json
    let meta_path = FsPath::new("packages/packages.json");
    let mut meta: HashMap<String, PackageMeta> = if meta_path.exists() {
        let raw = fs::read(meta_path).unwrap();
        let meta: HashMap<String, PackageMeta> = serde_json::from_slice(&raw).unwrap();
        migrate_meta(meta)
    } else {
        HashMap::new()
    };
    let entry = meta.entry(pkg.clone()).or_insert(PackageMeta {
        name: pkg.clone(),
        versions: HashMap::new(),
    });
    entry.versions.insert(
        version.clone(),
        VersionMeta {
            description: desc.clone(),
            yanked: false,
            deprecated: false,
            deprecation_message: None,
        },
    );
    fs::write(meta_path, serde_json::to_vec_pretty(&meta).unwrap()).ok();
    (
        StatusCode::OK,
        format!("Published package: {pkg}@{version}"),
    )
}

async fn get_metadata(Path(pkg): Path<String>) -> impl IntoResponse {
    let meta_path = FsPath::new("packages/packages.json");
    if !meta_path.exists() {
        return (StatusCode::NOT_FOUND, "No metadata".to_string());
    }
    let raw = fs::read(meta_path).unwrap();
    let meta: HashMap<String, PackageMeta> = serde_json::from_slice(&raw).unwrap();
    let meta = migrate_meta(meta);
    if let Some(pkg_meta) = meta.get(&pkg) {
        (
            StatusCode::OK,
            serde_json::to_string_pretty(pkg_meta).unwrap(),
        )
    } else {
        (StatusCode::NOT_FOUND, "Package not found".to_string())
    }
}

async fn get_tarball(Path((pkg, version)): Path<(String, String)>) -> Response {
    let path = format!("packages/{pkg}/{version}/package.tgz");
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
    let raw = fs::read(meta_path).unwrap();
    let meta: HashMap<String, PackageMeta> = serde_json::from_slice(&raw).unwrap();
    let meta = migrate_meta(meta);
    (StatusCode::OK, serde_json::to_string_pretty(&meta).unwrap())
}

async fn yank_version(Path((pkg, version)): Path<(String, String)>) -> impl IntoResponse {
    let meta_path = FsPath::new("packages/packages.json");
    let mut meta: HashMap<String, PackageMeta> = if meta_path.exists() {
        serde_json::from_slice(&fs::read(meta_path).unwrap()).unwrap()
    } else {
        return (StatusCode::NOT_FOUND, "No metadata".to_string());
    };
    if let Some(pkg_meta) = meta.get_mut(&pkg) {
        if let Some(ver_meta) = pkg_meta.versions.get_mut(&version) {
            ver_meta.yanked = true;
            fs::write(meta_path, serde_json::to_vec_pretty(&meta).unwrap()).ok();
            return (StatusCode::OK, format!("Yanked {pkg}@{version}"));
        }
    }
    (
        StatusCode::NOT_FOUND,
        "Package/version not found".to_string(),
    )
}

async fn unyank_version(Path((pkg, version)): Path<(String, String)>) -> impl IntoResponse {
    let meta_path = FsPath::new("packages/packages.json");
    let mut meta: HashMap<String, PackageMeta> = if meta_path.exists() {
        serde_json::from_slice(&fs::read(meta_path).unwrap()).unwrap()
    } else {
        return (StatusCode::NOT_FOUND, "No metadata".to_string());
    };
    if let Some(pkg_meta) = meta.get_mut(&pkg) {
        if let Some(ver_meta) = pkg_meta.versions.get_mut(&version) {
            ver_meta.yanked = false;
            fs::write(meta_path, serde_json::to_vec_pretty(&meta).unwrap()).ok();
            return (StatusCode::OK, format!("Unyanked {pkg}@{version}"));
        }
    }
    (
        StatusCode::NOT_FOUND,
        "Package/version not found".to_string(),
    )
}

#[derive(Deserialize)]
struct DeprecateReq {
    message: Option<String>,
}

async fn deprecate_version(
    Path((pkg, version)): Path<(String, String)>,
    Json(req): Json<DeprecateReq>,
) -> impl IntoResponse {
    let meta_path = FsPath::new("packages/packages.json");
    let mut meta: HashMap<String, PackageMeta> = if meta_path.exists() {
        serde_json::from_slice(&fs::read(meta_path).unwrap()).unwrap()
    } else {
        return (StatusCode::NOT_FOUND, "No metadata".to_string());
    };
    if let Some(pkg_meta) = meta.get_mut(&pkg) {
        if let Some(ver_meta) = pkg_meta.versions.get_mut(&version) {
            ver_meta.deprecated = true;
            ver_meta.deprecation_message = req.message;
            fs::write(meta_path, serde_json::to_vec_pretty(&meta).unwrap()).ok();
            return (StatusCode::OK, format!("Deprecated {pkg}@{version}"));
        }
    }
    (
        StatusCode::NOT_FOUND,
        "Package/version not found".to_string(),
    )
}

async fn search_packages(
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let q = params
        .get("q")
        .map(|s| s.to_lowercase())
        .unwrap_or_default();
    let meta_path = FsPath::new("packages/packages.json");
    if !meta_path.exists() {
        return (StatusCode::OK, "[]".to_string());
    }
    let raw = fs::read(meta_path).unwrap();
    let meta: HashMap<String, PackageMeta> = serde_json::from_slice(&raw).unwrap();
    let meta = migrate_meta(meta);
    let mut results = vec![];
    for pkg in meta.values() {
        if pkg.name.to_lowercase().contains(&q)
            || pkg
                .versions
                .values()
                .any(|v| v.description.to_lowercase().contains(&q))
        {
            results.push(pkg);
        }
    }
    (
        StatusCode::OK,
        serde_json::to_string_pretty(&results).unwrap(),
    )
}
