use rayon::prelude::*;
use std::fs;
use std::path::Path;

use crate::model::{is_os_critical_path, BuildingCategory, BuildingNode, SortMode};

pub const DEFAULT_MAX_DEPTH: usize = 6;

pub fn is_virtual_pseudo_fs(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    let trimmed = path_str.trim();

    // Linux virtual pseudo filesystems
    const UNIX_PSEUDO_FS: &[&str] = &["/proc", "/sys", "/dev", "/run"];
    for &pseudo in UNIX_PSEUDO_FS {
        if trimmed == pseudo || trimmed.starts_with(&format!("{}/", pseudo)) {
            return true;
        }
    }

    // Windows virtual / metadata paths
    let lower = trimmed.to_lowercase();
    if lower.contains("system volume information")
        || lower.ends_with("$recycle.bin")
        || lower.contains("$recycle.bin\\")
        || lower.contains("$winreagent")
    {
        return true;
    }

    false
}

pub struct ScannerOptions {
    pub max_depth: usize,
    pub sort_mode: SortMode,
}

impl Default for ScannerOptions {
    fn default() -> Self {
        Self {
            max_depth: DEFAULT_MAX_DEPTH,
            sort_mode: SortMode::SizeDesc,
        }
    }
}

pub fn scan_directory(
    root: &Path,
    options: &ScannerOptions,
) -> Result<BuildingNode, std::io::Error> {
    scan_node_recursive(root, 0, options.max_depth, options.sort_mode)
}

fn scan_node_recursive(
    path: &Path,
    current_depth: usize,
    max_depth: usize,
    sort_mode: SortMode,
) -> Result<BuildingNode, std::io::Error> {
    if is_virtual_pseudo_fs(path) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Virtual pseudo-filesystem bypassed",
        ));
    }

    let symlink_metadata = fs::symlink_metadata(path)?;
    let is_symlink = symlink_metadata.file_type().is_symlink();

    let raw_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string());

    let is_hidden = raw_name.starts_with('.');
    let is_system_protected = is_os_critical_path(path);
    let is_dir = !is_symlink && symlink_metadata.is_dir();

    if !is_dir {
        let size = symlink_metadata.len();
        let file_type = BuildingCategory::from_path(path, false, is_hidden);
        return Ok(BuildingNode {
            name: raw_name,
            path: path.to_path_buf(),
            is_dir: false,
            is_hidden,
            is_system_protected,
            size,
            file_type,
            children: Vec::new(),
        });
    }

    // It is a directory
    let entries = match fs::read_dir(path) {
        Ok(read_dir) => {
            let mut paths = Vec::new();
            for entry in read_dir.flatten() {
                let p = entry.path();
                if !is_virtual_pseudo_fs(&p) {
                    paths.push(p);
                }
            }
            paths
        }
        Err(_) => {
            // Permission denied or unreadable directory
            let file_type = BuildingCategory::from_path(path, true, is_hidden);
            return Ok(BuildingNode {
                name: raw_name,
                path: path.to_path_buf(),
                is_dir: true,
                is_hidden,
                is_system_protected,
                size: 0,
                file_type,
                children: Vec::new(),
            });
        }
    };

    let children: Vec<BuildingNode> = if current_depth >= max_depth {
        // Depth limit reached: calculate shallow size without building deeper child trees
        Vec::new()
    } else if current_depth == 0 && entries.len() > 8 {
        // Parallel scan for top-level entries when there are multiple items
        entries
            .par_iter()
            .filter_map(|child_path| {
                scan_node_recursive(child_path, current_depth + 1, max_depth, sort_mode).ok()
            })
            .collect()
    } else {
        // Sequential scan for deeper levels
        entries
            .iter()
            .filter_map(|child_path| {
                scan_node_recursive(child_path, current_depth + 1, max_depth, sort_mode).ok()
            })
            .collect()
    };

    let total_size: u64 = if current_depth >= max_depth {
        calculate_dir_size_fast(path)
    } else {
        children.iter().map(|c| c.size).sum()
    };

    let mut node = BuildingNode {
        name: raw_name,
        path: path.to_path_buf(),
        is_dir: true,
        is_hidden,
        is_system_protected,
        size: total_size,
        file_type: BuildingCategory::from_path(path, true, is_hidden),
        children,
    };

    node.sort_children(sort_mode);
    Ok(node)
}

/// Fast recursive size calculation when depth limit is reached, avoiding storing child nodes
fn calculate_dir_size_fast(dir: &Path) -> u64 {
    let mut total = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if is_virtual_pseudo_fs(&path) {
                continue;
            }
            if let Ok(meta) = fs::symlink_metadata(&path) {
                if meta.file_type().is_symlink() {
                    total += meta.len();
                } else if meta.is_dir() {
                    total += calculate_dir_size_fast(&path);
                } else {
                    total += meta.len();
                }
            }
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_is_virtual_pseudo_fs() {
        assert!(is_virtual_pseudo_fs(Path::new("/proc")));
        assert!(is_virtual_pseudo_fs(Path::new("/proc/cpuinfo")));
        assert!(is_virtual_pseudo_fs(Path::new("/sys")));
        assert!(is_virtual_pseudo_fs(Path::new("/dev/sda")));
        assert!(is_virtual_pseudo_fs(Path::new("/run/user/1000")));
        assert!(is_virtual_pseudo_fs(Path::new(
            "C:\\System Volume Information"
        )));
        assert!(is_virtual_pseudo_fs(Path::new("C:\\$Recycle.Bin")));

        assert!(!is_virtual_pseudo_fs(Path::new("/home/user")));
        assert!(!is_virtual_pseudo_fs(Path::new("/etc")));
        assert!(!is_virtual_pseudo_fs(Path::new("C:\\Users\\Admin")));
    }

    #[test]
    fn test_scanner_with_fixture_directory() {
        let temp_dir = std::env::temp_dir().join("disk_skyline_test_fixture");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(temp_dir.join("sub_code")).unwrap();
        fs::create_dir_all(temp_dir.join(".hidden_bunker")).unwrap();

        // Create files
        let mut f1 = File::create(temp_dir.join("sub_code").join("main.rs")).unwrap();
        f1.write_all(&[b'A'; 500]).unwrap();

        let mut f2 = File::create(temp_dir.join(".hidden_bunker").join("secret.key")).unwrap();
        f2.write_all(&[b'B'; 300]).unwrap();

        let mut f3 = File::create(temp_dir.join("root_doc.txt")).unwrap();
        f3.write_all(&[b'C'; 200]).unwrap();

        // Scan directory
        let options = ScannerOptions {
            max_depth: 4,
            sort_mode: SortMode::SizeDesc,
        };
        let root_node = scan_directory(&temp_dir, &options).unwrap();

        // Assertions
        assert!(root_node.is_dir);
        assert_eq!(root_node.size, 1000); // 500 + 300 + 200
        assert_eq!(root_node.children.len(), 3);

        // Find hidden bunker child
        let bunker = root_node
            .children
            .iter()
            .find(|c| c.name == ".hidden_bunker")
            .expect(".hidden_bunker must exist");
        assert!(bunker.is_hidden);
        assert_eq!(bunker.file_type, BuildingCategory::Restricted);
        assert_eq!(bunker.size, 300);

        // Find sub_code
        let sub_code = root_node
            .children
            .iter()
            .find(|c| c.name == "sub_code")
            .expect("sub_code must exist");
        assert_eq!(sub_code.file_type, BuildingCategory::Directory);
        assert_eq!(sub_code.size, 500);

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_scanner_depth_limit() {
        let temp_dir = std::env::temp_dir().join("disk_skyline_test_depth");
        let _ = fs::remove_dir_all(&temp_dir);
        let deep_path = temp_dir.join("d1").join("d2").join("d3");
        fs::create_dir_all(&deep_path).unwrap();

        let mut f = File::create(deep_path.join("deep.bin")).unwrap();
        f.write_all(&vec![b'X'; 1024]).unwrap();

        // Scan with max_depth 1 (only root and d1 direct items)
        let options = ScannerOptions {
            max_depth: 1,
            sort_mode: SortMode::SizeDesc,
        };
        let root = scan_directory(&temp_dir, &options).unwrap();

        // Root has 1 child: d1
        assert_eq!(root.children.len(), 1);
        let d1 = &root.children[0];
        assert_eq!(d1.name, "d1");
        assert_eq!(d1.size, 1024);
        assert_eq!(d1.children.len(), 0);

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
