use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PurgeAction {
    /// Execute a shell binary with arguments (e.g. `docker`, `["system", "prune", "-f", "--volumes"]`)
    Command(String, Vec<String>),
    /// Delete all contents within a directory
    DirectoryDelete(PathBuf),
    /// Remove inactive snap revisions (name, revision)
    SnapRevisions(Vec<(String, String)>),
}

impl PurgeAction {
    pub fn execute(&self) -> Result<(), String> {
        match self {
            PurgeAction::Command(cmd, args) => {
                let output = Command::new(cmd)
                    .args(args)
                    .output()
                    .map_err(|e| format!("Failed to run {}: {}", cmd, e))?;
                if output.status.success() {
                    Ok(())
                } else {
                    let err = String::from_utf8_lossy(&output.stderr);
                    Err(format!("{} error: {}", cmd, err.trim()))
                }
            }
            PurgeAction::DirectoryDelete(path) => {
                if path.exists() {
                    std::fs::remove_dir_all(path)
                        .map_err(|e| format!("Failed to delete {}: {}", path.display(), e))?;
                    // Re-create the empty directory so tools don't fail on missing cache root
                    let _ = std::fs::create_dir_all(path);
                }
                Ok(())
            }
            PurgeAction::SnapRevisions(revisions) => {
                let mut errors = Vec::new();
                for (name, rev) in revisions {
                    let status = Command::new("snap")
                        .args(["remove", name, "--revision", rev])
                        .status();
                    match status {
                        Ok(s) if s.success() => {}
                        Ok(s) => errors.push(format!(
                            "snap remove {} rev {} exited with {}",
                            name, rev, s
                        )),
                        Err(e) => errors.push(format!("Failed to run snap remove: {}", e)),
                    }
                }
                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(errors.join("; "))
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanableTarget {
    pub id: String,
    pub name: String,
    pub size: u64,
    pub description: String,
    pub purge_action: PurgeAction,
    pub requires_sudo: bool,
}

/// Helper to calculate directory size quickly (ignoring errors on inaccessible files)
pub fn calculate_dir_size(path: &Path) -> u64 {
    if !path.is_dir() {
        return path.metadata().map(|m| m.len()).unwrap_or(0);
    }
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                total = total.saturating_add(calculate_dir_size(&p));
            } else if let Ok(meta) = entry.metadata() {
                total = total.saturating_add(meta.len());
            }
        }
    }
    total
}
pub fn parse_size_string(s: &str) -> u64 {
    let s = s.trim();
    if s.is_empty() {
        return 0;
    }

    let (num_part, unit_raw) = if let Some(idx) = s.find(|c: char| c.is_alphabetic()) {
        (&s[..idx], &s[idx..])
    } else {
        (s, "")
    };

    let unit: String = unit_raw
        .chars()
        .take_while(|c| c.is_alphabetic())
        .collect::<String>()
        .to_uppercase();

    let val = match num_part.trim().parse::<f64>() {
        Ok(v) => v,
        Err(_) => return 0,
    };

    let multiplier: f64 = match unit.as_str() {
        "B" | "BYTES" => 1.0,
        "KB" | "K" | "KIB" => 1024.0,
        "MB" | "M" | "MIB" => 1024.0 * 1024.0,
        "GB" | "G" | "GIB" => 1024.0 * 1024.0 * 1024.0,
        "TB" | "T" | "TIB" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => 1.0,
    };

    (val * multiplier).round() as u64
}

/// Probe Docker for dangling images, stopped containers, and unused volumes
pub fn probe_docker() -> Option<CleanableTarget> {
    let output = Command::new("docker")
        .args(["system", "df", "--format", "{{json .}}"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut total_reclaimable: u64 = 0;
    let mut total_items = 0usize;

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // Minimal json extraction without heavy dependencies
        // Look for "Reclaimable":"..."
        if let Some(pos) = line.find("\"Reclaimable\":\"") {
            let rest = &line[pos + 15..];
            if let Some(end) = rest.find('\"') {
                let rec_str = &rest[..end];
                // "2.039GB (66%)" -> take string before parenthesis
                let size_str = rec_str.split('(').next().unwrap_or("").trim();
                let bytes = parse_size_string(size_str);
                if bytes > 0 {
                    total_reclaimable = total_reclaimable.saturating_add(bytes);
                    total_items += 1;
                }
            }
        }
    }

    if total_reclaimable > 0 {
        Some(CleanableTarget {
            id: "docker".into(),
            name: "Docker Dangling & Volumes".into(),
            size: total_reclaimable,
            description: format!("{} unused/reclaimable image & volume sets", total_items),
            purge_action: PurgeAction::Command(
                "docker".into(),
                vec![
                    "system".into(),
                    "prune".into(),
                    "-f".into(),
                    "--volumes".into(),
                ],
            ),
            requires_sudo: false,
        })
    } else {
        None
    }
}

/// Probe Snap for disabled revisions
pub fn probe_snap() -> Option<CleanableTarget> {
    let output = Command::new("snap").args(["list", "--all"]).output().ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut disabled_revisions = Vec::new();
    let mut total_size: u64 = 0;

    for line in text.lines().skip(1) {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() >= 6 && cols.last().map(|s| s.contains("disabled")).unwrap_or(false) {
            let name = cols[0].to_string();
            let rev = cols[2].to_string();
            // Estimate size from /var/lib/snapd/snaps/{name}_{rev}.snap
            let snap_file = PathBuf::from(format!("/var/lib/snapd/snaps/{}_{}.snap", name, rev));
            let file_size = snap_file
                .metadata()
                .map(|m| m.len())
                .unwrap_or(150 * 1024 * 1024); // fallback 150MB
            total_size = total_size.saturating_add(file_size);
            disabled_revisions.push((name, rev));
        }
    }

    if !disabled_revisions.is_empty() {
        let count = disabled_revisions.len();
        Some(CleanableTarget {
            id: "snap".into(),
            name: "Snap Inactive Revisions".into(),
            size: total_size,
            description: format!("{} old disabled package revisions", count),
            purge_action: PurgeAction::SnapRevisions(disabled_revisions),
            requires_sudo: true,
        })
    } else {
        None
    }
}

/// Probe package manager caches in user home
pub fn probe_package_caches() -> Vec<CleanableTarget> {
    let mut targets = Vec::new();
    let home = match std::env::var("HOME") {
        Ok(h) => PathBuf::from(h),
        Err(_) => return targets,
    };

    // 1. NPM Cache
    let npm_cache = home.join(".npm/_cacache");
    if npm_cache.is_dir() {
        let size = calculate_dir_size(&npm_cache);
        if size > 10 * 1024 * 1024 {
            // > 10 MB
            targets.push(CleanableTarget {
                id: "npm".into(),
                name: "NPM Package Cache".into(),
                size,
                description: "Global ~/.npm/_cacache package artifacts".into(),
                purge_action: PurgeAction::DirectoryDelete(npm_cache),
                requires_sudo: false,
            });
        }
    }

    // 2. Python Pip Cache
    let pip_cache = home.join(".cache/pip");
    if pip_cache.is_dir() {
        let size = calculate_dir_size(&pip_cache);
        if size > 5 * 1024 * 1024 {
            targets.push(CleanableTarget {
                id: "pip".into(),
                name: "Python Pip Cache".into(),
                size,
                description: "Cached wheels and downloads in ~/.cache/pip".into(),
                purge_action: PurgeAction::DirectoryDelete(pip_cache),
                requires_sudo: false,
            });
        }
    }

    // 3. Python uv Cache
    let uv_cache = home.join(".cache/uv");
    if uv_cache.is_dir() {
        let size = calculate_dir_size(&uv_cache);
        if size > 5 * 1024 * 1024 {
            targets.push(CleanableTarget {
                id: "uv".into(),
                name: "Python uv Cache".into(),
                size,
                description: "Cached packages and wheels in ~/.cache/uv".into(),
                purge_action: PurgeAction::DirectoryDelete(uv_cache),
                requires_sudo: false,
            });
        }
    }

    // 3. Cargo Registry Cache
    let cargo_cache = home.join(".cargo/registry/cache");
    if cargo_cache.is_dir() {
        let size = calculate_dir_size(&cargo_cache);
        if size > 10 * 1024 * 1024 {
            targets.push(CleanableTarget {
                id: "cargo".into(),
                name: "Cargo Crate Archives Cache".into(),
                size,
                description: "Downloaded .crate archives in ~/.cargo/registry/cache".into(),
                purge_action: PurgeAction::DirectoryDelete(cargo_cache),
                requires_sudo: false,
            });
        }
    }

    // 4. Playwright Browser Cache
    let playwright_cache = home.join(".cache/ms-playwright");
    if playwright_cache.is_dir() {
        let size = calculate_dir_size(&playwright_cache);
        if size > 50 * 1024 * 1024 {
            targets.push(CleanableTarget {
                id: "playwright".into(),
                name: "Playwright Test Browsers".into(),
                size,
                description: "Automated test browser builds in ~/.cache/ms-playwright".into(),
                purge_action: PurgeAction::DirectoryDelete(playwright_cache),
                requires_sudo: false,
            });
        }
    }

    // 5. NuGet Package Cache
    let nuget_cache = home.join(".nuget/packages");
    if nuget_cache.is_dir() {
        let size = calculate_dir_size(&nuget_cache);
        if size > 50 * 1024 * 1024 {
            targets.push(CleanableTarget {
                id: "nuget".into(),
                name: "NuGet Package Cache".into(),
                size,
                description: "Extracted .nupkg packages in ~/.nuget/packages".into(),
                purge_action: PurgeAction::DirectoryDelete(nuget_cache),
                requires_sudo: false,
            });
        }
    }

    targets
}

/// Probe Apt archives
pub fn probe_apt_archives() -> Option<CleanableTarget> {
    let apt_cache = PathBuf::from("/var/cache/apt/archives");
    if apt_cache.is_dir() {
        let size = calculate_dir_size(&apt_cache);
        if size > 20 * 1024 * 1024 {
            // > 20MB
            return Some(CleanableTarget {
                id: "apt".into(),
                name: "Apt Package Archives".into(),
                size,
                description: "Cached .deb installer archives in /var/cache/apt".into(),
                purge_action: PurgeAction::Command("apt-get".into(), vec!["clean".into()]),
                requires_sudo: true,
            });
        }
    }
    None
}

/// Main survey orchestrator gathering all available cleanable junk
pub fn survey_all_system_junk() -> Vec<CleanableTarget> {
    let mut targets = Vec::new();

    if let Some(docker) = probe_docker() {
        targets.push(docker);
    }
    if let Some(snap) = probe_snap() {
        targets.push(snap);
    }
    targets.extend(probe_package_caches());
    if let Some(apt) = probe_apt_archives() {
        targets.push(apt);
    }

    // Sort descending by reclaimable size
    targets.sort_by_key(|a| std::cmp::Reverse(a.size));
    targets
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size_string() {
        assert_eq!(parse_size_string("500B"), 500);
        assert_eq!(parse_size_string("2KB"), 2048);
        assert_eq!(parse_size_string("1.5MB"), (1.5 * 1024.0 * 1024.0) as u64);
        assert_eq!(
            parse_size_string("2.039GB (66%)"),
            (2.039 * 1024.0 * 1024.0 * 1024.0) as u64
        );
        assert_eq!(parse_size_string("0B"), 0);
        assert_eq!(parse_size_string(""), 0);
    }

    #[test]
    fn test_calculate_dir_size_nonexistent() {
        assert_eq!(calculate_dir_size(Path::new("/nonexistent_path_12345")), 0);
    }

    #[test]
    fn test_purge_action_directory_delete() {
        let temp_dir = std::env::temp_dir().join("test_disk_skyline_purge_dir");
        let _ = std::fs::create_dir_all(&temp_dir);
        let test_file = temp_dir.join("temp.txt");
        let _ = std::fs::write(&test_file, b"test content");
        assert!(test_file.exists());

        let action = PurgeAction::DirectoryDelete(temp_dir.clone());
        let res = action.execute();
        assert!(res.is_ok());
        assert!(!test_file.exists());
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_survey_all_system_junk_resilience() {
        let targets = survey_all_system_junk();
        for t in &targets {
            assert!(!t.id.is_empty());
            assert!(!t.name.is_empty());
            assert!(t.size > 0);
        }
    }
}
