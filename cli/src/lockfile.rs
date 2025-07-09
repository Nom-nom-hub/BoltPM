use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;

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
}

pub fn write_lockfile(path: &Path, lock: &BoltLock) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(lock)?;
    fs::write(path.join("bolt.lock"), json)?;
    Ok(())
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
        };
        let tmp_dir = tempfile::tempdir().unwrap();
        write_lockfile(tmp_dir.path(), &lock).unwrap();
        assert!(tmp_dir.path().join("bolt.lock").exists());
    }
} 