use clap::{Parser, Subcommand, CommandFactory};
use colored::*;
use std::fs;
use serde::{Deserialize, Serialize};
mod lockfile;
use std::path::Path;
mod plugin;
use crate::plugin::run_plugins;
use plugin_api::PluginContext;
use log::{error, info};
use std::env;
use std::fs::{create_dir_all, write};
use std::path::PathBuf;
use reqwest::Client;
use tokio::runtime::Runtime;
use std::collections::{HashSet, BTreeMap};
use glob::glob;

#[derive(Parser)]
#[command(
    name = "BoltPM",
    version = "0.1.0",
    about = "⚡️ BoltPM — Fast, Modern NPM Alternative",
    long_about = "A blazing fast, extensible, and workspace-friendly package manager for JavaScript/TypeScript monorepos.\n\nDocs: https://github.com/yourusername/BoltPM",
    after_help = "✨ For more info, see: https://github.com/yourusername/BoltPM\n"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
    #[arg(long)]
    pub frozen_lockfile: bool,
    #[arg(long, default_value = "info")]
    pub log_level: String,
    #[arg(short, long)]
    pub help: bool,
    #[arg(long)]
    pub version: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Initialize a new BoltPM project")]
    Init,
    #[command(about = "Install dependencies")]
    Install,
    #[command(about = "Remove a package")]
    Remove,
    #[command(about = "Update dependencies")]
    Update,
    #[command(about = "Run a script")]
    Run,
    #[command(about = "Link a local package")]
    Link,
    #[command(about = "Yank a package from the registry")]
    Yank,
    #[command(about = "Restore a yanked package")]
    Unyank,
    #[command(about = "Mark a package as deprecated")]
    Deprecate,
    #[command(about = "Search for packages")]
    Search,
    #[command(about = "Generate or update the lockfile")]
    Lock,
    #[command(about = "Plugin management")]
    Plugin,
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
    #[serde(default)]
    pub workspaces: Vec<String>,
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

// Workspace helpers (scaffold)
fn is_workspace_root(lock: &BoltLock) -> bool {
    !lock.workspaces.is_empty()
}

fn enumerate_workspace_packages(lock: &BoltLock) -> Vec<PathBuf> {
    let mut packages = Vec::new();
    for pattern in &lock.workspaces {
        match glob(pattern) {
            Ok(paths) => {
                for entry in paths.flatten() {
                    if entry.is_dir() && entry.join("package.json").exists() {
                        packages.push(entry.canonicalize().unwrap_or(entry));
                    }
                }
            }
            Err(e) => {
                eprintln!("[WORKSPACE] Invalid glob pattern '{}': {}", pattern, e);
            }
        }
    }
    packages
}

fn find_workspace_root(start: &Path) -> Option<PathBuf> {
    let mut cur = start.canonicalize().ok()?;
    loop {
        let lock_path = cur.join("bolt.lock");
        if lock_path.exists() {
            if let Ok(s) = std::fs::read_to_string(&lock_path) {
                if let Ok(lock) = serde_json::from_str::<BoltLock>(&s) {
                    if !lock.workspaces.is_empty() {
                        return Some(cur);
                    }
                }
            }
        }
        if !cur.pop() { break; }
    }
    None
}

fn parse_package_json(path: &Path) -> Option<PackageJson> {
    let pj_path = path.join("package.json");
    let pj_str = std::fs::read_to_string(&pj_path).ok()?;
    serde_json::from_str(&pj_str).ok()
}

fn run_workspace_plugins(pkg_path: &Path, hook: &str, package_name: &str, package_version: &str, output_path: &str) {
    use crate::plugin::run_plugins;
    use plugin_api::PluginContext;
    use std::collections::HashMap;
    // Prefer local .boltpm/plugins, else fallback to workspace root
    let local_plugins = pkg_path.join(".boltpm/plugins");
    let ws_root = find_workspace_root(pkg_path).unwrap_or_else(|| pkg_path.to_path_buf());
    let plugins_dir = if local_plugins.exists() { local_plugins } else { ws_root.join(".boltpm/plugins") };
    if !plugins_dir.exists() {
        return;
    }
    let ctx = PluginContext {
        hook: hook.to_string(),
        package_name: package_name.to_string(),
        package_version: package_version.to_string(),
        install_path: pkg_path.canonicalize().unwrap().to_string_lossy().to_string(),
        output_path: output_path.to_string(),
        env: std::env::vars().collect::<HashMap<_, _>>(),
    };
    if let Err(e) = run_plugins(hook, &ctx) {
        eprintln!("[WORKSPACE PLUGIN] Plugin hook '{}' failed for {}: {}", hook, package_name, e);
    }
}

fn install_workspace_package(
    pkg_path: &Path,
    store_root: &Path,
    global_lock: &mut BoltLock,
    installed: &mut HashSet<(String, String)>,
    ws_root: &Path,
) {
    let pj = match parse_package_json(pkg_path) {
        Some(pj) => pj,
        None => {
            eprintln!("[WORKSPACE] Failed to parse package.json at {}", pkg_path.display());
            return;
        }
    };
    let name = pj.name.clone();
    let version = pj.version.clone();
    let key = (name.clone(), version.clone());
    if installed.contains(&key) {
        // Already installed (deduped)
        return;
    }
    installed.insert(key.clone());
    // Install this package into the global store if not present
    let store_pkg_dir = store_root.join(&name).join(&version);
    if !store_pkg_dir.exists() {
        std::fs::create_dir_all(&store_pkg_dir).expect("Failed to create store dir");
        // Copy all files from pkg_path to store_pkg_dir (simulate extract)
        for entry in std::fs::read_dir(pkg_path).unwrap() {
            let entry = entry.unwrap();
            let src = entry.path();
            let dst = store_pkg_dir.join(src.file_name().unwrap());
            if src.is_file() {
                std::fs::copy(&src, &dst).unwrap();
            } else if src.is_dir() {
                // Recursively copy subdirs (simple, not robust)
                let _ = std::process::Command::new("cp").args(["-R", src.to_str().unwrap(), dst.to_str().unwrap()]).status();
            }
        }
    }
    // Link into node_modules of this workspace package
    let node_modules = pkg_path.join("node_modules");
    std::fs::create_dir_all(&node_modules).unwrap();
    let link_path = node_modules.join(&name);
    if link_path.exists() {
        let _ = std::fs::remove_file(&link_path);
        let _ = std::fs::remove_dir_all(&link_path);
    }
    #[cfg(target_family = "unix")]
    std::os::unix::fs::symlink(&store_pkg_dir, &link_path).unwrap();
    #[cfg(target_family = "windows")]
    std::os::windows::fs::symlink_dir(&store_pkg_dir, &link_path).unwrap();
    // Recursively install dependencies
    if let Some(deps) = pj.dependencies {
        if let Some(map) = deps.as_object() {
            for (dep, _ver) in map {
                // For demo, assume dependency is another workspace package if present
                let dep_path = pkg_path.parent().unwrap().join(dep);
                if dep_path.join("package.json").exists() {
                    install_workspace_package(&dep_path, store_root, global_lock, installed, ws_root);
                } else {
                    // TODO: Fetch from registry if not local
                    eprintln!("[WORKSPACE] External dep {} not implemented", dep);
                }
            }
        }
    }
    // After install/link, run postinstall plugin hooks for this package
    // Compute output_path: <ws_root>/.boltpm/plugins_output/<package-name>/
    let output_path = ws_root.join(".boltpm/plugins_output").join(&name);
    std::fs::create_dir_all(&output_path).unwrap();
    println!("🔧 Running plugins for workspace package: {}", pkg_path.display());
    println!("  install_path: {}", pkg_path.canonicalize().unwrap().display());
    println!("  output_path: {}", output_path.display());
    run_workspace_plugins(pkg_path, "postinstall", &name, &version, &output_path.to_string_lossy());
    println!("✅ Finished plugins for {}", pkg_path.display());
}

fn handle_list(remote: bool) -> Result<(), Box<dyn std::error::Error>> {
    if remote {
        let registry_url = env::var("BOLTPM_REGISTRY_URL").unwrap_or_else(|_| "http://localhost:4000/v1/plugins".to_string());
        let url = format!("{}", registry_url);
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let client = Client::new();
            match client.get(&url).timeout(std::time::Duration::from_secs(10)).send().await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        let plugins: serde_json::Value = resp.json().await.unwrap_or_default();
                        println!("Remote plugins:");
                        if let Some(arr) = plugins.as_array() {
                            for p in arr {
                                let name = p["name"].as_str().unwrap_or("");
                                let version = p["version"].as_str().unwrap_or("");
                                let desc = p["description"].as_str().unwrap_or("");
                                let trust = p["trust_level"].as_str().unwrap_or("unknown");
                                println!("- {}@{}: {} [trust: {}]", name, version, desc, trust);
                            }
                        } else {
                            println!("No plugins found.");
                        }
                    } else {
                        println!("Registry error: {}", resp.status());
                    }
                }
                Err(e) => println!("Network error: {}", e),
            }
        });
    } else {
        let cwd = env::current_dir()?;
        let plugins_dir = cwd.join(".boltpm/plugins");
        println!("Installed plugins in {}:", plugins_dir.display());
        if !plugins_dir.exists() {
            println!("  (No plugins installed)");
            return Ok(());
        }
        let mut found = false;
        for entry in fs::read_dir(&plugins_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                println!("  {}", path.file_name().unwrap().to_string_lossy());
                found = true;
            }
        }
        if !found {
            println!("  (No plugins installed)");
        }
    }
    Ok(())
}

fn handle_uninstall(name: String) -> Result<(), Box<dyn std::error::Error>> {
    let cwd = env::current_dir()?;
    let plugins_dir = cwd.join(".boltpm/plugins");
    let mut found = false;
    if plugins_dir.exists() {
        for entry in fs::read_dir(&plugins_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.file_name().unwrap().to_string_lossy().contains(&name) {
                println!("Uninstalling plugin: {}", path.display());
                fs::remove_file(&path)?;
                found = true;
            }
        }
    }
    if !found {
        println!("Plugin '{}' not found in {}", name, plugins_dir.display());
    }
    Ok(())
}

fn main() {
    let cli = Cli::parse();
    if cli.version {
        println!("{}", "BoltPM v0.1.0".bold().yellow());
        return;
    }
    if cli.help {
        print_help();
        return;
    }

    // Initialize logging after parsing CLI arguments, using CLI log_level as default
    let env = env_logger::Env::default()
        .default_filter_or(&cli.log_level);
    env_logger::init_from_env(env);

    // Set log level based on CLI argument with proper error handling
    match cli.log_level.parse() {
        Ok(level) => {
            log::set_max_level(level);
        }
        Err(e) => {
            eprintln!("Invalid log level '{}': {}. Please use one of: error, warn, info, debug, trace.", cli.log_level, e);
            std::process::exit(1);
        }
    }

    info!("BoltPM starting up");
    
    match cli.command {
        Some(Commands::Init) => {
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
        Some(Commands::Install) => {
            info!("Installing package: {:?}", package);
            let mut lock = read_lockfile();
            let cwd = std::env::current_dir().unwrap();
            // Workspace install logic
            if package.is_none() {
                if let Some(ws_root) = find_workspace_root(&cwd) {
                    let store_root = ws_root.join(".boltpm/store");
                    let ws_lock = read_lockfile();
                    let ws_packages = enumerate_workspace_packages(&ws_lock);
                    // After discovering workspace packages:
                    println!("Discovered workspace packages:");
                    for pkg_path in &ws_packages {
                        println!("- {}", pkg_path.display());
                    }
                    let mut installed = HashSet::new();
                    for pkg_path in ws_packages {
                        install_workspace_package(&pkg_path, &store_root, &mut lock, &mut installed, &ws_root);
                    }
                    write_lockfile(&lock);
                    info!("Workspace install complete.");
                    return;
                }
            }
            // Parse package.json
            let pj_str = fs::read_to_string("package.json").expect("No package.json found");
            let pj: PackageJson = serde_json::from_str(&pj_str).expect("Invalid package.json");
            info!("Parsed package.json: {:?}", pj);
            let mut changed = false;
            // Plugin context setup
            let ctx = PluginContext {
                hook: "preinstall".to_string(),
                package_name: pj.name.clone(),
                package_version: pj.version.clone(),
                install_path: std::env::current_dir().unwrap().canonicalize().unwrap().to_string_lossy().to_string(),
                output_path: std::env::current_dir().unwrap().join(".boltpm/plugins_output").to_string_lossy().to_string(),
                env: std::env::vars().collect(),
            };
            // Always run preinstall plugins, even if no dependencies or fetch fails
            if let Err(e) = run_plugins("preinstall", &ctx) {
                error!("Preinstall plugin failed: {}", e);
                std::process::exit(1);
            }
            // Check for frozen lockfile mismatch
            if cli.frozen_lockfile {
                let mut mismatched = false;
                if let Some(deps) = &pj.dependencies {
                    if let Some(map) = deps.as_object() {
                        for dep in map.keys() {
                            if !lock.packages.contains_key(dep) {
                                error!("Dependency '{}' in package.json missing from bolt.lock", dep);
                                mismatched = true;
                            }
                        }
                    }
                }
                for dep in lock.packages.keys() {
                    if let Some(deps) = &pj.dependencies {
                        if let Some(map) = deps.as_object() {
                            if !map.contains_key(dep) {
                                error!("Package '{}' in bolt.lock missing from package.json", dep);
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
                    install_path: std::env::current_dir().unwrap().canonicalize().unwrap().to_string_lossy().to_string(),
                    output_path: std::env::current_dir().unwrap().join(".boltpm/plugins_output").to_string_lossy().to_string(),
                    env: std::env::vars().collect(),
                };
                // Run preinstall plugins for this package
                if let Err(e) = run_plugins("preinstall", &ctx) {
                    error!("Preinstall plugin failed: {}", e);
                    std::process::exit(1);
                }
                // If already in lockfile, use pinned version
                if let Some(entry) = lock.packages.get(pkg) {
                    info!("Using {}@{} from lockfile", pkg, entry.version);
                    // Run postinstall plugins for this package
                    let ctx_post = PluginContext {
                        hook: "postinstall".to_string(),
                        package_name: pkg.to_string(),
                        package_version: entry.version.clone(),
                        install_path: entry.resolved.clone(),
                        output_path: entry.resolved.clone(),
                        env: std::env::vars().collect(),
                    };
                    if let Err(e) = run_plugins("postinstall", &ctx_post) {
                        error!("Postinstall plugin failed: {}", e);
                        std::process::exit(1);
                    }
                    return;
                }
                let url = format!("http://localhost:4000/v1/{}/", pkg);
                info!("Fetching metadata from {}", url);
                let meta_resp = reqwest::blocking::get(&url);
                match meta_resp {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            let meta: serde_json::Value = resp.json().unwrap();
                            let versions = meta["versions"].as_object().unwrap();
                            let latest = versions.keys().last().unwrap();
                            let tarball_url = format!("http://localhost:4000/v1/{}/{}/", pkg, latest);
                            info!("Downloading tarball from {}", tarball_url);
                            let tarball_resp = reqwest::blocking::get(&tarball_url).unwrap();
                            if tarball_resp.status().is_success() {
                                let bytes = tarball_resp.bytes().unwrap();
                                let cache_dir = format!(".boltpm/cache/{}-{}", pkg, latest);
                                fs::create_dir_all(&cache_dir).unwrap();
                                let tarball_path = format!("{}/package.tgz", cache_dir);
                                fs::write(&tarball_path, &bytes).unwrap();
                                // PluginContext for hooks
                                let ctx = PluginContext {
                                    hook: "preinstall".to_string(),
                                    package_name: pkg.to_string(),
                                    package_version: latest.to_string(),
                                    install_path: cache_dir.clone(),
                                    output_path: cache_dir.clone(),
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
                                    println!("Extraction failed: {}", e);
                                    let _ = run_plugins("onError", &ctx);
                                    return;
                                }
                                info!("Extracted to {}", cache_dir);
                                // Recursively install dependencies if package.json exists in extracted dir
                                let extracted_pj = format!("{}/package.json", cache_dir);
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
                                    error!("Postinstall plugin failed: {}", e);
                                    std::process::exit(1);
                                }
                                info!("Install complete: {}@{}", pkg, latest);
                            } else {
                                error!("Failed to download tarball: {}", tarball_resp.status());
                                let ctx = PluginContext {
                                    hook: "preinstall".to_string(),
                                    package_name: pkg.to_string(),
                                    package_version: "unknown".to_string(),
                                    install_path: std::path::PathBuf::from("").to_string_lossy().to_string(),
                                    output_path: std::path::PathBuf::from("").to_string_lossy().to_string(),
                                    env: std::env::vars().collect(),
                                };
                                let _ = run_plugins("onError", &ctx);
                            }
                        } else {
                            error!("Failed to fetch metadata: {}", resp.status());
                        }
                    }
                    Err(e) => {
                        error!("Error fetching metadata: {}", e);
                    }
                }
                // Always run postinstall plugins for this package
                let ctx_post = PluginContext {
                    hook: "postinstall".to_string(),
                    package_name: pkg.to_string(),
                    package_version: "unknown".to_string(),
                    install_path: std::env::current_dir().unwrap().canonicalize().unwrap().to_string_lossy().to_string(),
                    output_path: std::env::current_dir().unwrap().join(".boltpm/plugins_output").to_string_lossy().to_string(),
                    env: std::env::vars().collect(),
                };
                if let Err(e) = run_plugins("postinstall", &ctx_post) {
                    error!("Postinstall plugin failed: {}", e);
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
                install_path: std::env::current_dir().unwrap().canonicalize().unwrap().to_string_lossy().to_string(),
                output_path: std::env::current_dir().unwrap().join(".boltpm/plugins_output").to_string_lossy().to_string(),
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
        Some(Commands::Remove) => {
            info!("Removing package: {}", package);
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
            info!("Removed {} and its dependencies from bolt.lock.", package);
            // TODO: Remove from node_modules, filesystem, etc.
            // TODO: Call onError plugin hook if needed
        }
        Some(Commands::Update) => {
            info!("Updating package: {:?}", package);
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
                let url = format!("http://localhost:4000/v1/{}/", pkg);
                info!("Fetching metadata from {}", url);
                let meta_resp = reqwest::blocking::get(&url);
                match meta_resp {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            let meta: serde_json::Value = resp.json().unwrap();
                            let versions = meta["versions"].as_object().unwrap();
                            let latest = versions.keys().last().unwrap();
                            let tarball_url = format!("http://localhost:4000/v1/{}/{}/", pkg, latest);
                            info!("Downloading tarball from {}", tarball_url);
                            let tarball_resp = reqwest::blocking::get(&tarball_url).unwrap();
                            if tarball_resp.status().is_success() {
                                let bytes = tarball_resp.bytes().unwrap();
                                let cache_dir = format!(".boltpm/cache/{}-{}", pkg, latest);
                                fs::create_dir_all(&cache_dir).unwrap();
                                let tarball_path = format!("{}/package.tgz", cache_dir);
                                fs::write(&tarball_path, &bytes).unwrap();
                                // PluginContext for hooks
                                let ctx = PluginContext {
                                    hook: "preinstall".to_string(),
                                    package_name: pkg.to_string(),
                                    package_version: latest.to_string(),
                                    install_path: cache_dir.clone(),
                                    output_path: cache_dir.clone(),
                                    env: std::env::vars().collect(),
                                };
                                let _ = run_plugins("preinstall", &ctx);
                                let tar_gz = fs::File::open(&tarball_path).unwrap();
                                let decompressed = flate2::read::GzDecoder::new(tar_gz);
                                let mut archive = tar::Archive::new(decompressed);
                                if let Err(e) = archive.unpack(&cache_dir) {
                                    println!("Extraction failed: {}", e);
                                    let _ = run_plugins("onError", &ctx);
                                    return;
                                }
                                info!("Extracted to {}", cache_dir);
                                let extracted_pj = format!("{}/package.json", cache_dir);
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
                                info!("Update complete: {}@{}", pkg, latest);
                            } else {
                                error!("Failed to download tarball: {}", tarball_resp.status());
                                let ctx = PluginContext {
                                    hook: "preinstall".to_string(),
                                    package_name: pkg.to_string(),
                                    package_version: latest.to_string(),
                                    install_path: std::path::PathBuf::from("").to_string_lossy().to_string(),
                                    output_path: std::path::PathBuf::from("").to_string_lossy().to_string(),
                                    env: std::env::vars().collect(),
                                };
                                let _ = run_plugins("onError", &ctx);
                            }
                        } else {
                            error!("Failed to fetch metadata: {}", resp.status());
                        }
                    }
                    Err(e) => {
                        error!("Error fetching metadata: {}", e);
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
        Some(Commands::Run) => {
            info!("Running script: {} (stub)", script);
            // TODO: Run script from package.json
        }
        Some(Commands::Link) => {
            info!("Linking path: {:?} (stub)", path);
            // TODO: Link local package
        }
        Some(Commands::Yank) => {
            let url = format!("http://localhost:4000/v1/{}/{}/yank", package, version);
            let resp = reqwest::blocking::Client::new().post(&url).send();
            match resp {
                Ok(r) => println!("{}", r.text().unwrap()),
                Err(e) => println!("Error: {}", e),
            }
        }
        Some(Commands::Unyank) => {
            let url = format!("http://localhost:4000/v1/{}/{}/unyank", package, version);
            let resp = reqwest::blocking::Client::new().post(&url).send();
            match resp {
                Ok(r) => println!("{}", r.text().unwrap()),
                Err(e) => println!("Error: {}", e),
            }
        }
        Some(Commands::Deprecate) => {
            let url = format!("http://localhost:4000/v1/{}/{}/deprecate", package, version);
            let body = serde_json::json!({ "message": message });
            let resp = reqwest::blocking::Client::new().post(&url).json(&body).send();
            match resp {
                Ok(r) => println!("{}", r.text().unwrap()),
                Err(e) => println!("Error: {}", e),
            }
        }
        Some(Commands::Search) => {
            let url = format!("http://localhost:4000/v1/search?q={}", urlencoding::encode(&query));
            let resp = reqwest::blocking::get(&url);
            match resp {
                Ok(r) => {
                    let text = r.text().unwrap();
                    println!("Search results: {}", text);
                }
                Err(e) => println!("Error: {}", e),
            }
        }
        Some(Commands::Lock) => {
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
                workspaces: vec![],
            };
            lockfile::write_lockfile(Path::new("."), &lock).expect("Failed to write bolt.lock");
            println!("bolt.lock generated.");
        }
        Some(Commands::Plugin) => {
            match command {
                PluginCommand::Init { name } => {
                    use std::fs::{create_dir_all, write};
                    use std::path::PathBuf;
                    let base = PathBuf::from("wasm_plugins").join(&name);
                    let src = base.join("src");
                    let cargo = base.join("Cargo.toml");
                    let lib = src.join("lib.rs");
                    let cargo_config = base.join(".cargo/config.toml");
                    create_dir_all(&src).expect("Failed to create plugin src dir");
                    create_dir_all(base.join(".cargo")).expect("Failed to create .cargo dir");
                    // Cargo.toml
                    let cargo_toml = format!(r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
"#);
                    write(&cargo, cargo_toml).expect("Failed to write Cargo.toml");
                    // .cargo/config.toml
                    let config_toml = r#"[build]
target = "wasm32-unknown-unknown"
"#;
                    write(&cargo_config, config_toml).expect("Failed to write .cargo/config.toml");
                    // src/lib.rs
                    let lib_rs = r#"// Minimal BoltPM WASM plugin (WASI + host logging)

#[no_mangle]
pub extern "C" fn _boltpm_plugin_v1() {}

#[no_mangle]
pub extern "C" fn _run() {
    // WASI stdout
    println!("[PLUGIN] Hello from WASI stdout!");

    // Host logging function
    let msg = "[PLUGIN] Hello from host logging!";
    unsafe {
        boltpm_log(msg.as_ptr(), msg.len());
    }
}

extern "C" {
    fn boltpm_log(ptr: *const u8, len: usize);
}
"#;
                    write(&lib, lib_rs).expect("Failed to write lib.rs");
                    println!("✅ WASM plugin scaffolded at {}", base.display());
                    println!("To build: cd {} && cargo build --release --target wasm32-unknown-unknown", base.display());
                }
                PluginCommand::Build { name } => {
                    use std::process::Command;
                    use std::fs;
                    use std::path::PathBuf;
                    let mut plugins = Vec::new();
                    let base = PathBuf::from("wasm_plugins");
                    if let Some(name) = name {
                        let dir = base.join(&name);
                        if dir.exists() && dir.join("Cargo.toml").exists() {
                            plugins.push((name.to_string(), dir));
                        } else {
                            eprintln!("❌ Plugin '{}' not found or missing Cargo.toml", name);
                            std::process::exit(1);
                        }
                    } else {
                        if let Ok(entries) = fs::read_dir(&base) {
                            for entry in entries.flatten() {
                                let path = entry.path();
                                if path.is_dir() && path.join("Cargo.toml").exists() {
                                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                        plugins.push((name.to_string(), path));
                                    }
                                }
                            }
                        }
                        if plugins.is_empty() {
                            println!("No WASM plugins found in wasm_plugins/");
                            return;
                        }
                    }
                    let mut any_failed = false;
                    for (name, dir) in plugins {
                        println!("🔨 Building plugin: {}", name);
                        let status = Command::new("cargo")
                            .arg("build")
                            .arg("--release")
                            .arg("--target")
                            .arg("wasm32-unknown-unknown")
                            .current_dir(&dir)
                            .env("RUSTFLAGS", std::env::var("RUSTFLAGS").unwrap_or_default())
                            .status();
                        match status {
                            Ok(s) if s.success() => {
                                let wasm_path = dir.join("target/wasm32-unknown-unknown/release").join(format!("{}.wasm", name));
                                if wasm_path.exists() {
                                    println!("✅ Built {} → {}", name, wasm_path.display());
                                } else {
                                    println!("⚠️  Build succeeded but .wasm not found: {}", wasm_path.display());
                                    any_failed = true;
                                }
                            }
                            Ok(s) => {
                                println!("❌ Build failed for {} (exit code {})", name, s);
                                any_failed = true;
                            }
                            Err(e) => {
                                println!("❌ Build error for {}: {}", name, e);
                                any_failed = true;
                            }
                        }
                    }
                    if any_failed {
                        println!("Some plugins failed to build.");
                        std::process::exit(1);
                    } else {
                        println!("All plugins built successfully.");
                    }
                }
                PluginCommand::Run { name } => {
                    #[cfg(feature = "wasm-support")]
                    {
                        use wasmtime::{Engine, Module, Store, Linker};
                        use wasmtime_wasi::WasiCtxBuilder;
                        use std::path::PathBuf;
                        let wasm_path = PathBuf::from("wasm_plugins").join(&name).join("target/wasm32-unknown-unknown/release").join(format!("{}.wasm", name));
                        if !wasm_path.exists() {
                            eprintln!("❌ WASM file not found: {}", wasm_path.display());
                            std::process::exit(1);
                        }
                        let engine = Engine::default();
                        let module = match Module::from_file(&engine, &wasm_path) {
                            Ok(m) => m,
                            Err(e) => {
                                eprintln!("❌ Failed to load WASM module: {}", e);
                                std::process::exit(1);
                            }
                        };
                        let mut linker = Linker::new(&engine);
                        linker.func_wrap("env", "boltpm_log", |mut caller: wasmtime::Caller<'_, wasmtime_wasi::WasiCtx>, ptr: i32, len: i32| {
                            let mem = caller.get_export("memory").and_then(|e| e.into_memory());
                            if let Some(mem) = mem {
                                let data = mem.data(&caller)[ptr as usize..(ptr+len) as usize].to_vec();
                                if let Ok(msg) = String::from_utf8(data) {
                                    println!("[WASM PLUGIN LOG] {}", msg);
                                }
                            }
                        }).unwrap();
                        let mut wasi_builder = WasiCtxBuilder::new().inherit_stdio();
                        wasi_builder = match wasi_builder.inherit_env() {
                            Ok(w) => w,
                            Err(e) => {
                                eprintln!("❌ Failed to inherit env: {}", e);
                                std::process::exit(1);
                            }
                        };
                        let wasi = wasi_builder.build();
                        let mut store = Store::new(&engine, wasi);
                        wasmtime_wasi::add_to_linker(&mut linker, |s| s).unwrap();
                        let instance = match linker.instantiate(&mut store, &module) {
                            Ok(i) => i,
                            Err(e) => {
                                eprintln!("❌ Failed to instantiate WASM module: {}", e);
                                std::process::exit(1);
                            }
                        };
                        if instance.get_func(&mut store, "_boltpm_plugin_v1").is_none() {
                            eprintln!("❌ WASM plugin missing required ABI version export _boltpm_plugin_v1");
                            std::process::exit(1);
                        }
                        let run_func = match instance.get_func(&mut store, "_run") {
                            Some(f) => f,
                            None => {
                                eprintln!("❌ WASM plugin missing required '_run' function");
                                std::process::exit(1);
                            }
                        };
                        match run_func.call(&mut store, &[], &mut []) {
                            Ok(_) => {
                                println!("✅ Plugin '{}' ran successfully.", name);
                            }
                            Err(e) => {
                                eprintln!("❌ Plugin '{}' failed: {}", name, e);
                                std::process::exit(1);
                            }
                        }
                    }
                    #[cfg(not(feature = "wasm-support"))]
                    {
                        eprintln!("WASM support is not enabled. Cannot run WASM plugins.");
                        std::process::exit(1);
                    }
                }
                PluginCommand::Search { query } => {
                    let registry_url = env::var("BOLTPM_REGISTRY_URL").unwrap_or_else(|_| "http://localhost:4000/v1/plugins".to_string());
                    let url = format!("{}/search?q={}", registry_url, urlencoding::encode(&query));
                    let rt = Runtime::new().unwrap();
                    rt.block_on(async {
                        let client = Client::new();
                        match client.get(&url).timeout(std::time::Duration::from_secs(10)).send().await {
                            Ok(resp) => {
                                if resp.status().is_success() {
                                    let plugins: serde_json::Value = resp.json().await.unwrap_or_default();
                                    println!("Remote plugins matching '{}':", query);
                                    if let Some(arr) = plugins.as_array() {
                                        for p in arr {
                                            let name = p["name"].as_str().unwrap_or("");
                                            let version = p["version"].as_str().unwrap_or("");
                                            let desc = p["description"].as_str().unwrap_or("");
                                            let trust = p["trust_level"].as_str().unwrap_or("unknown");
                                            println!("- {}@{}: {} [trust: {}]", name, version, desc, trust);
                                        }
                                    } else {
                                        println!("No plugins found.");
                                    }
                                } else {
                                    println!("Registry error: {}", resp.status());
                                }
                            }
                            Err(e) => println!("Network error: {}", e),
                        }
                    });
                }
                PluginCommand::List { remote } => {
                    if let Err(e) = handle_list(remote) {
                        eprintln!("Error listing plugins: {e}");
                        std::process::exit(1);
                    }
                }
                PluginCommand::Install { plugin } => {
                    // Parse plugin name and optional version
                    let (name, version) = if let Some((n, v)) = plugin.split_once('@') {
                        (n.to_string(), Some(v.to_string()))
                    } else {
                        (plugin.clone(), None)
                    };
                    let registry_url = env::var("BOLTPM_REGISTRY_URL").unwrap_or_else(|_| "http://localhost:4000/v1/plugins".to_string());
                    let meta_url = if let Some(ref v) = version {
                        format!("{}/{}/{}", registry_url, name, v)
                    } else {
                        format!("{}/{}", registry_url, name)
                    };
                    let rt = Runtime::new().unwrap();
                    rt.block_on(async {
                        let client = Client::new();
                        println!("Fetching plugin metadata from {}...", meta_url);
                        match client.get(&meta_url).timeout(std::time::Duration::from_secs(10)).send().await {
                            Ok(resp) => {
                                if resp.status().is_success() {
                                    let meta: serde_json::Value = resp.json().await.unwrap_or_default();
                                    let download_url = meta["download_url"].as_str().unwrap_or("");
                                    let expected_hash = meta["sha256"].as_str().unwrap_or("");
                                    let abi_version = meta["abi_version"].as_str().unwrap_or("");
                                    if download_url.is_empty() {
                                        println!("No download URL in plugin metadata.");
                                        return;
                                    }
                                    println!("Downloading plugin from {}...", download_url);
                                    match client.get(download_url).timeout(std::time::Duration::from_secs(30)).send().await {
                                        Ok(bin_resp) => {
                                            if bin_resp.status().is_success() {
                                                let bytes = bin_resp.bytes().await.unwrap();
                                                use sha2::{Digest, Sha256};
                                                let mut hasher = Sha256::new();
                                                hasher.update(&bytes);
                                                let hash = format!("{:x}", hasher.finalize());
                                                if !expected_hash.is_empty() && hash != expected_hash {
                                                    println!("Hash mismatch! Expected {}, got {}", expected_hash, hash);
                                                    return;
                                                }
                                                // Save to .boltpm/plugins
                                                let plugins_dir = PathBuf::from(".boltpm/plugins");
                                                fs::create_dir_all(&plugins_dir).unwrap();
                                                let ext = if download_url.ends_with(".wasm") { "wasm" } else { "dylib" };
                                                let filename = format!("{}.{}", name, ext);
                                                let plugin_path = plugins_dir.join(&filename);
                                                fs::write(&plugin_path, &bytes).unwrap();
                                                println!("Plugin saved to {}", plugin_path.display());
                                                // WASM ABI check: require _boltpm_plugin_v1 export
                                                let abi_check = if plugin_path.extension().map(|e| e == "wasm").unwrap_or(false) {
                                                    let wasm_bytes = match std::fs::read(&plugin_path) {
                                                        Ok(b) => b,
                                                        Err(e) => {
                                                            println!("[ERROR] Failed to read plugin file: {}", e);
                                                            std::fs::remove_file(&plugin_path).ok();
                                                            return;
                                                        }
                                                    };
                                                    let mut has_abi = false;
                                                    for payload in wasmparser::Parser::new(0).parse_all(&wasm_bytes) {
                                                        if let Ok(wasmparser::Payload::ExportSection(s)) = payload {
                                                            for export in s {
                                                                let export = match export {
                                                                    Ok(e) => e,
                                                                    Err(e) => {
                                                                        println!("[ERROR] Failed to parse WASM export: {}", e);
                                                                        break;
                                                                    }
                                                                };
                                                                if export.name == "_boltpm_plugin_v1" {
                                                                    has_abi = true;
                                                                    break;
                                                                }
                                                            }
                                                        }
                                                        if has_abi { break; }
                                                    }
                                                    if !has_abi {
                                                        println!("[ERROR] Plugin is missing required ABI export: _boltpm_plugin_v1");
                                                        std::fs::remove_file(&plugin_path).ok();
                                                        return;
                                                    }
                                                    true
                                                } else {
                                                    // TODO: Native plugin ABI check using object crate
                                                    true
                                                };
                                                if !abi_check {
                                                    return;
                                                }
                                                // TODO: Native plugin ABI check using object crate
                                                println!("✅ Plugin {} installed successfully!", name);
                                            } else {
                                                println!("Download failed: {}", bin_resp.status());
                                            }
                                        }
                                        Err(e) => println!("Download error: {}", e),
                                    }
                                } else {
                                    println!("Registry error: {}", resp.status());
                                }
                            }
                            Err(e) => println!("Network error: {}", e),
                        }
                    });
                }
                PluginCommand::Uninstall { name } => {
                    if let Err(e) = handle_uninstall(name) {
                        eprintln!("Error uninstalling plugin: {e}");
                        std::process::exit(1);
                    }
                }
            }
        }
    }
} 