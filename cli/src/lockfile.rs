use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct LockDependency {
    pub version: String,
    pub resolved: String,
    pub integrity: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BoltLock {
    pub name: String,
    pub dependencies: HashMap<String, LockDependency>,
    #[serde(default)]
    pub workspaces: Vec<String>,
}

pub fn write_lockfile(path: &Path, lock: &BoltLock) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(lock)?;
    fs::write(path.join("bolt.lock"), json)?;
    Ok(())
}

pub fn enumerate_workspace_packages(lock: &BoltLock) -> Vec<PathBuf> {
    println!("[debug] CWD: {}", std::env::current_dir().unwrap().display());
    println!("[debug] Workspaces defined in lockfile:");
    for w in &lock.workspaces {
        println!("  - {}", w);
    }
    let mut packages = Vec::new();
    for pattern in &lock.workspaces {
        println!("[debug] Expanding pattern: {}", pattern);
        for entry in glob::glob(pattern).unwrap() {
            match entry {
                Ok(path) => {
                    println!("[debug] -> matched: {}", path.display());
                    if path.join("package.json").exists() {
                        packages.push(path.canonicalize().unwrap());
                    }
                },
                Err(e) => println!("[debug] -> glob error: {:?}", e),
            }
        }
    }
    packages
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_lockfile_serialization() {
        let mut deps = HashMap::new();
        deps.insert("lodash".into(), LockDependency {
            version: "4.17.21".into(),
            resolved: "https://registry.npmjs.org/lodash/-/lodash-4.17.21.tgz".into(),
            integrity: None,
        });
        let lock = BoltLock {
            name: "test-app".into(),
            dependencies: deps,
            workspaces: vec!["packages/app".into(), "packages/api".into()],
        };
        let tmp_dir = tempfile::tempdir().unwrap();
        write_lockfile(tmp_dir.path(), &lock).unwrap();
        assert!(tmp_dir.path().join("bolt.lock").exists());
    }
} 