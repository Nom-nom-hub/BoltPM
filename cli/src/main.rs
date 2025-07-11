use clap::{Parser, Subcommand};
use std::fs;
use serde::{Deserialize, Serialize};
mod lockfile;
use std::path::Path;
mod plugin;
use crate::plugin::run_plugins;
use plugin_api::PluginContext;
use log::{error, info};

#[derive(Parser)]
#[command(name = "boltpm")]
#[command(about = "BoltPM: Fast, modern NPM alternative", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(long, default_value_t = false)]
    frozen_lockfile: bool,
    #[arg(long, default_value = "info")]
    log_level: String,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Install {
        package: Option<String>,
    },
    Remove {
        package: String,
    },
    Update {
        package: Option<String>,
    },
    Run {
        script: String,
    },
    Link {
        path: Option<String>,
    },
    Yank {
        package: String,
        version: String,
    },
    Unyank {
        package: String,
        version: String,
    },
    Deprecate {
        package: String,
        version: String,
        message: Option<String>,
    },
    Search {
        query: String,
    },
    Lock, // <-- add this
}

#[derive(Serialize, Deserialize, Debug)]
struct PackageJson {
    name: String,
    version: String,
    dependencies: Option<serde_json::Value>,
    // ... more fields as needed
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct BoltLock {
    pub packages: std::collections::BTreeMap<String, BoltLockEntry>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct BoltLockEntry {
    pub version: String,
    pub resolved: String,
    pub dependencies: Option<std::collections::BTreeMap<String, String>>, // dep name -> version
}

fn read_lockfile() -> BoltLock {
    match fs::read_to_string("bolt.lock") {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => BoltLock::default(),
    }
}

fn write_lockfile(lock: &BoltLock) {
    let s = serde_json::to_string_pretty(lock).unwrap();
    fs::write("bolt.lock", s).expect("Failed to write bolt.lock");
}

fn main() {
    let cli = Cli::parse();

    // Initialize logging after parsing CLI arguments, using CLI log_level as default
    let env = env_logger::Env::default()
        .default_filter_or(&cli.log_level);
    env_logger::init_from_env(env);

    // Logging is initialized above using env_logger and the CLI log_level as default.

    info!("BoltPM starting up");
    
    match cli.command {
        Commands::Init => {
            info!("Initializing new BoltPM project...");
            let pj = PackageJson {
                name: "my-boltpm-project".to_string(),
                version: "0.1.0".to_string(),
                dependencies: None,
            };
            let pj_str = serde_json::to_string_pretty(&pj).unwrap();
            fs::write("package.json", pj_str).expect("Failed to write package.json");
            fs::create_dir_all(".boltpm").expect("Failed to create .boltpm dir");
            fs::write("bolt.lock", "{}\n").expect("Failed to write bolt.lock");
            info!("Project initialized successfully");
        }
        Commands::Install { package } => {
            info!("Installing package: {package:?}");
            // Parse package.json
            let pj_str = fs::read_to_string("package.json").expect("No package.json found");
            let pj: PackageJson = serde_json::from_str(&pj_str).expect("Invalid package.json");
            info!("Parsed package.json: {pj:?}");
            let mut lock = read_lockfile();
            let mut changed = false;
            // Plugin context setup
            let ctx = PluginContext {
                hook: "preinstall".to_string(),
                package_name: pj.name.clone(),
                package_version: pj.version.clone(),
                install_path: std::env::current_dir().unwrap().to_string_lossy().to_string(),
                env: std::env::vars().collect(),
            };
            // Always run preinstall plugins, even if no dependencies or fetch fails
            if let Err(e) = run_plugins("preinstall", &ctx) {
                error!("Preinstall plugin failed: {e}");
                std::process::exit(1);
            }
            // Check for frozen lockfile mismatch
            if cli.frozen_lockfile {
                let mut mismatched = false;
                if let Some(deps) = &pj.dependencies {
                    if let Some(map) = deps.as_object() {
                        for dep in map.keys() {
                            if !lock.packages.contains_key(dep) {
                                error!("Dependency '{dep}' in package.json missing from bolt.lock");
                                mismatched = true;
                            }
                        }
                    }
                }
                for dep in lock.packages.keys() {
                    if let Some(deps) = &pj.dependencies {
                        if let Some(map) = deps.as_object() {
                            if !map.contains_key(dep) {
                                error!("Package '{dep}' in bolt.lock missing from package.json");
                                mismatched = true;
                            }
                        }
                    }
                }
                if mismatched {
                    error!("bolt.lock and package.json are out of sync. Aborting due to --frozen-lockfile.");
                    std::process::exit(1);
                }
            }
            fn install_pkg(pkg: &str, lock: &mut BoltLock, changed: &mut bool) {
                // Plugin context for this package
                let ctx = PluginContext {
                    hook: "preinstall".to_string(),
                    package_name: pkg.to_string(),
                    package_version: "unknown".to_string(),
                    install_path: std::env::current_dir().unwrap().to_string_lossy().to_string(),
                    env: std::env::vars().collect(),
                };
                // Run preinstall plugins for this package
                if let Err(e) = run_plugins("preinstall", &ctx) {
                    error!("Preinstall plugin failed: {e}");
                    std::process::exit(1);
                }
                // If already in lockfile, use pinned version
                if let Some(entry) = lock.packages.get(pkg) {
                    info!("Using {pkg}@{version} from lockfile", version=entry.version);
                    // Run postinstall plugins for this package
                    let ctx_post = PluginContext {
                        hook: "postinstall".to_string(),
                        package_name: pkg.to_string(),
                        package_version: entry.version.clone(),
                        install_path: entry.resolved.clone(),
                        env: std::env::vars().collect(),
                    };
                    if let Err(e) = run_plugins("postinstall", &ctx_post) {
                        error!("Postinstall plugin failed: {e}");
                        std::process::exit(1);
                    }
                    return;
                }
                let url = format!("http://localhost:4000/v1/{pkg}/");
                info!("Fetching metadata from {url}");
                let meta_resp = reqwest::blocking::get(&url);
                match meta_resp {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            let meta: serde_json::Value = resp.json().unwrap();
                            let versions = meta["versions"].as_object().unwrap();
                            let latest = versions.keys().next_back().unwrap();
                            let tarball_url = format!("http://localhost:4000/v1/{pkg}/{latest}/");
                            info!("Downloading tarball from {tarball_url}");
                            let tarball_resp = reqwest::blocking::get(&tarball_url).unwrap();
                            if tarball_resp.status().is_success() {
                                let bytes = tarball_resp.bytes().unwrap();
                                let cache_dir = format!(".boltpm/cache/{pkg}-{latest}");
                                fs::create_dir_all(&cache_dir).unwrap();
                                let tarball_path = format!("{cache_dir}/package.tgz");
                                fs::write(&tarball_path, &bytes).unwrap();
                                // PluginContext for hooks
                                let ctx = PluginContext {
                                    hook: "preinstall".to_string(),
                                    package_name: pkg.to_string(),
                                    package_version: latest.to_string(),
                                    install_path: cache_dir.clone(),
                                    env: std::env::vars().collect(),
                                };
                                // Call preinstall plugin hook
                                if let Err(e) = run_plugins("preinstall", &ctx) {
                                    error!("Preinstall plugin failed: {}", e);
                                    std::process::exit(1);
                                }
                                // Extract tarball
                                let tar_gz = fs::File::open(&tarball_path).unwrap();
                                let decompressed = flate2::read::GzDecoder::new(tar_gz);
                                let mut archive = tar::Archive::new(decompressed);
                                if let Err(e) = archive.unpack(&cache_dir) {
                                    println!("Extraction failed: {e}");
                                    let _ = run_plugins("onError", &ctx);
                                    return;
                                }
                                info!("Extracted to {cache_dir}");
                                // Recursively install dependencies if package.json exists in extracted dir
                                let extracted_pj = format!("{cache_dir}/package.json");
                                let mut dependencies = std::collections::BTreeMap::new();
                                if let Ok(dep_pj_str) = fs::read_to_string(&extracted_pj) {
                                    if let Ok(dep_pj) = serde_json::from_str::<PackageJson>(&dep_pj_str) {
                                        if let Some(deps) = dep_pj.dependencies {
                                            if let Some(map) = deps.as_object() {
                                                for (dep, ver) in map {
                                                    dependencies.insert(dep.clone(), ver.as_str().unwrap_or("").to_string());
                                                    install_pkg(dep, lock, changed);
                                                }
                                            }
                                        }
                                    }
                                }
                                // Update lockfile
                                lock.packages.insert(pkg.to_string(), BoltLockEntry {
                                    version: latest.to_string(),
                                    resolved: tarball_url.clone(),
                                    dependencies: if dependencies.is_empty() { None } else { Some(dependencies) },
                                });
                                *changed = true;
                                // Call postinstall plugin hook
                                if let Err(e) = run_plugins("postinstall", &ctx) {
                                    error!("Postinstall plugin failed: {e}");
                                    std::process::exit(1);
                                }
                                info!("Install complete: {pkg}@{latest}");
                            } else {
                                error!("Failed to download tarball: {status}", status=tarball_resp.status());
                                let ctx = PluginContext {
                                    hook: "preinstall".to_string(),
                                    package_name: pkg.to_string(),
                                    package_version: "unknown".to_string(),
                                    install_path: std::path::PathBuf::from("").to_string_lossy().to_string(),
                                    env: std::env::vars().collect(),
                                };
                                let _ = run_plugins("onError", &ctx);
                            }
                        } else {
                            error!("Failed to fetch metadata: {status}", status=resp.status());
                        }
                    }
                    Err(e) => {
                        error!("Error fetching metadata: {e}");
                    }
                }
                // Always run postinstall plugins for this package
                let ctx_post = PluginContext {
                    hook: "postinstall".to_string(),
                    package_name: pkg.to_string(),
                    package_version: "unknown".to_string(),
                    install_path: std::env::current_dir().unwrap().to_string_lossy().to_string(),
                    env: std::env::vars().collect(),
                };
                if let Err(e) = run_plugins("postinstall", &ctx_post) {
                    error!("Postinstall plugin failed: {e}");
                    std::process::exit(1);
                }
            }
            if let Some(pkg) = package {
                install_pkg(&pkg, &mut lock, &mut changed);
            } else {
                // Install all dependencies from package.json
                if let Some(deps) = pj.dependencies {
                    if let Some(map) = deps.as_object() {
                        for (dep, _ver) in map {
                            install_pkg(dep, &mut lock, &mut changed);
                        }
                    }
                } else {
                    info!("No dependencies to install.");
                }
            }
            // After install, run postinstall plugins (always)
            let ctx_post = PluginContext {
                hook: "postinstall".to_string(),
                package_name: pj.name.clone(),
                package_version: pj.version.clone(),
                install_path: std::env::current_dir().unwrap().to_string_lossy().to_string(),
                env: std::env::vars().collect(),
            };
            if let Err(e) = run_plugins("postinstall", &ctx_post) {
                error!("Postinstall plugin failed: {}", e);
                std::process::exit(1);
            }
            if changed {
                write_lockfile(&lock);
                info!("bolt.lock updated.");
            } else {
                info!("No changes to bolt.lock.");
            }
        }
        Commands::Remove { package } => {
            info!("Removing package: {package}");
            let mut lock = read_lockfile();
            // Remove the package and its dependencies recursively from lockfile
            fn remove_pkg(pkg: &str, lock: &mut BoltLock) {
                if let Some(entry) = lock.packages.remove(pkg) {
                    if let Some(deps) = entry.dependencies {
                        for dep in deps.keys() {
                            remove_pkg(dep, lock);
                        }
                    }
                }
            }
            remove_pkg(&package, &mut lock);
            write_lockfile(&lock);
            info!("Removed {package} and its dependencies from bolt.lock.");
            // TODO: Remove from node_modules, filesystem, etc.
            // TODO: Call onError plugin hook if needed
        }
        Commands::Update { package } => {
            info!("Updating package: {package:?}");
            let pj_str = fs::read_to_string("package.json").expect("No package.json found");
            let pj: PackageJson = serde_json::from_str(&pj_str).expect("Invalid package.json");
            let mut lock = read_lockfile();
            let mut changed = false;
            fn update_pkg(pkg: &str, lock: &mut BoltLock, changed: &mut bool) {
                // Remove old entry if exists
                lock.packages.remove(pkg);
                // Reinstall to get latest and update lockfile
                // (reuse install_pkg logic from install command)
                // For now, duplicate logic for clarity
                let url = format!("http://localhost:4000/v1/{pkg}/");
                info!("Fetching metadata from {url}");
                let meta_resp = reqwest::blocking::get(&url);
                match meta_resp {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            let meta: serde_json::Value = resp.json().unwrap();
                            let versions = meta["versions"].as_object().unwrap();
                            let latest = versions.keys().next_back().unwrap();
                            let tarball_url = format!("http://localhost:4000/v1/{pkg}/{latest}/");
                            info!("Downloading tarball from {tarball_url}");
                            let tarball_resp = reqwest::blocking::get(&tarball_url).unwrap();
                            if tarball_resp.status().is_success() {
                                let bytes = tarball_resp.bytes().unwrap();
                                let cache_dir = format!(".boltpm/cache/{pkg}-{latest}");
                                fs::create_dir_all(&cache_dir).unwrap();
                                let tarball_path = format!("{cache_dir}/package.tgz");
                                fs::write(&tarball_path, &bytes).unwrap();
                                // PluginContext for hooks
                                let ctx = PluginContext {
                                    hook: "preinstall".to_string(),
                                    package_name: pkg.to_string(),
                                    package_version: latest.to_string(),
                                    install_path: cache_dir.clone(),
                                    env: std::env::vars().collect(),
                                };
                                let _ = run_plugins("preinstall", &ctx);
                                let tar_gz = fs::File::open(&tarball_path).unwrap();
                                let decompressed = flate2::read::GzDecoder::new(tar_gz);
                                let mut archive = tar::Archive::new(decompressed);
                                if let Err(e) = archive.unpack(&cache_dir) {
                                    println!("Extraction failed: {e}");
                                    let _ = run_plugins("onError", &ctx);
                                    return;
                                }
                                info!("Extracted to {cache_dir}");
                                let extracted_pj = format!("{cache_dir}/package.json");
                                let mut dependencies = std::collections::BTreeMap::new();
                                if let Ok(dep_pj_str) = fs::read_to_string(&extracted_pj) {
                                    if let Ok(dep_pj) = serde_json::from_str::<PackageJson>(&dep_pj_str) {
                                        if let Some(deps) = dep_pj.dependencies {
                                            if let Some(map) = deps.as_object() {
                                                for (dep, ver) in map {
                                                    dependencies.insert(dep.clone(), ver.as_str().unwrap_or("").to_string());
                                                    update_pkg(dep, lock, changed);
                                                }
                                            }
                                        }
                                    }
                                }
                                lock.packages.insert(pkg.to_string(), BoltLockEntry {
                                    version: latest.to_string(),
                                    resolved: tarball_url.clone(),
                                    dependencies: if dependencies.is_empty() { None } else { Some(dependencies) },
                                });
                                *changed = true;
                                let _ = run_plugins("postinstall", &ctx);
                                info!("Update complete: {pkg}@{latest}");
                            } else {
                                error!("Failed to download tarball: {status}", status=tarball_resp.status());
                                let ctx = PluginContext {
                                    hook: "preinstall".to_string(),
                                    package_name: pkg.to_string(),
                                    package_version: latest.to_string(),
                                    install_path: std::path::PathBuf::from("").to_string_lossy().to_string(),
                                    env: std::env::vars().collect(),
                                };
                                let _ = run_plugins("onError", &ctx);
                            }
                        } else {
                            error!("Failed to fetch metadata: {status}", status=resp.status());
                        }
                    }
                    Err(e) => {
                        error!("Error fetching metadata: {e}");
                    }
                }
            }
            if let Some(pkg) = package {
                update_pkg(&pkg, &mut lock, &mut changed);
            } else {
                // Update all dependencies from package.json
                if let Some(deps) = pj.dependencies {
                    if let Some(map) = deps.as_object() {
                        for (dep, _ver) in map {
                            update_pkg(dep, &mut lock, &mut changed);
                        }
                    }
                } else {
                    info!("No dependencies to update.");
                }
            }
            if changed {
                write_lockfile(&lock);
                info!("bolt.lock updated.");
            } else {
                info!("No changes to bolt.lock.");
            }
        }
        Commands::Run { script } => {
            info!("Running script: {script} (stub)");
            // TODO: Run script from package.json
        }
        Commands::Link { path } => {
            info!("Linking path: {path:?} (stub)");
            // TODO: Link local package
        }
        Commands::Yank { package, version } => {
            let url = format!("http://localhost:4000/v1/{package}/{version}/yank");
            let resp = reqwest::blocking::Client::new().post(&url).send();
            match resp {
                Ok(r) => println!("{}", r.text().unwrap()),
                Err(e) => println!("Error: {e}"),
            }
        }
        Commands::Unyank { package, version } => {
            let url = format!("http://localhost:4000/v1/{package}/{version}/unyank");
            let resp = reqwest::blocking::Client::new().post(&url).send();
            match resp {
                Ok(r) => println!("{}", r.text().unwrap()),
                Err(e) => println!("Error: {e}"),
            }
        }
        Commands::Deprecate { package, version, message } => {
            let url = format!("http://localhost:4000/v1/{package}/{version}/deprecate");
            let body = serde_json::json!({ "message": message });
            let resp = reqwest::blocking::Client::new().post(&url).json(&body).send();
            match resp {
                Ok(r) => println!("{}", r.text().unwrap()),
                Err(e) => println!("Error: {e}"),
            }
        }
        Commands::Search { query } => {
            let url = format!("http://localhost:4000/v1/search?q={}", urlencoding::encode(&query));
            let resp = reqwest::blocking::get(&url);
            match resp {
                Ok(r) => {
                    let text = r.text().unwrap();
                    println!("Search results: {text}");
                }
                Err(e) => println!("Error: {e}"),
            }
        }
        Commands::Lock => {
            // Read package.json (for now, just get the name)
            let pj_str = fs::read_to_string("package.json").expect("No package.json found");
            let pj: serde_json::Value = serde_json::from_str(&pj_str).expect("Invalid package.json");
            let name = pj["name"].as_str().unwrap_or("bolt-app").to_string();
            // Hardcode a dependency for demonstration
            let mut deps = std::collections::HashMap::new();
            deps.insert("lodash".to_string(), lockfile::LockDependency {
                version: "4.17.21".to_string(),
                resolved: "https://registry.npmjs.org/lodash/-/lodash-4.17.21.tgz".to_string(),
                integrity: None,
            });
            let lock = lockfile::BoltLock {
                name,
                dependencies: deps,
            };
            lockfile::write_lockfile(Path::new("."), &lock).expect("Failed to write bolt.lock");
            println!("bolt.lock generated.");
        }
    }
} 