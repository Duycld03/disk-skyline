use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PurgeAction {
    /// Execute a shell binary with arguments (e.g. `docker`, `["system", "prune", "-f", "--volumes"]`)
    Command(String, Vec<String>),
    /// Delete all contents within a directory
    DirectoryDelete(PathBuf),
    /// Delete multiple directories
    MultiDirectoryDelete(Vec<PathBuf>),
    /// Remove inactive snap revisions (name, revision)
    SnapRevisions(Vec<(String, String)>),
    /// Remove user-owned temporary files in /tmp
    TmpUserDebris(Vec<PathBuf>),
    /// Vacuum systemd journal logs to a target size (e.g. "100M")
    JournalVacuum(String),
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
            PurgeAction::MultiDirectoryDelete(paths) => {
                let mut errors = Vec::new();
                for path in paths {
                    if path.exists() {
                        if let Err(e) = std::fs::remove_dir_all(path) {
                            errors.push(format!("Failed to delete {}: {}", path.display(), e));
                        }
                    }
                }
                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(errors.join("; "))
                }
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
            PurgeAction::TmpUserDebris(paths) => {
                for p in paths {
                    if p.is_dir() {
                        let _ = std::fs::remove_dir_all(p);
                    } else if p.is_file() || p.is_symlink() {
                        let _ = std::fs::remove_file(p);
                    }
                }
                Ok(())
            }
            PurgeAction::JournalVacuum(size_arg) => {
                let vacuum_arg = format!("--vacuum-size={}", size_arg);
                let output = Command::new("journalctl")
                    .arg(&vacuum_arg)
                    .output()
                    .map_err(|e| format!("Failed to run journalctl: {}", e))?;
                if output.status.success() {
                    Ok(())
                } else {
                    let err = String::from_utf8_lossy(&output.stderr);
                    Err(format!("journalctl error: {}", err.trim()))
                }
            }
        }
    }

    pub fn execute_with_sudo(&self, password: &str) -> Result<(), String> {
        match self {
            PurgeAction::SnapRevisions(revisions) => {
                let mut errors = Vec::new();
                for (name, rev) in revisions {
                    if let Err(e) = run_sudo_command(
                        "snap",
                        &["remove", name.as_str(), "--revision", rev.as_str()],
                        password,
                    ) {
                        errors.push(e);
                    }
                }
                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(errors.join("; "))
                }
            }
            PurgeAction::Command(cmd, args) => {
                let str_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                run_sudo_command(cmd, &str_args, password)
            }
            PurgeAction::JournalVacuum(size_arg) => {
                let vacuum_arg = format!("--vacuum-size={}", size_arg);
                run_sudo_command("journalctl", &[&vacuum_arg], password)
            }
            PurgeAction::DirectoryDelete(_)
            | PurgeAction::MultiDirectoryDelete(_)
            | PurgeAction::TmpUserDebris(_) => self.execute(),
        }
    }

    pub fn requires_sudo(&self) -> bool {
        match self {
            PurgeAction::Command(cmd, _) => cmd == "apt-get" || cmd == "apt",
            PurgeAction::SnapRevisions(_) => true,
            PurgeAction::JournalVacuum(_) => true,
            PurgeAction::DirectoryDelete(_)
            | PurgeAction::MultiDirectoryDelete(_)
            | PurgeAction::TmpUserDebris(_) => false,
        }
    }

    pub fn manual_guidance(&self) -> Option<String> {
        match self {
            PurgeAction::SnapRevisions(revisions) => {
                if let Some((name, rev)) = revisions.first() {
                    Some(format!("sudo snap remove {} --revision {}", name, rev))
                } else {
                    Some("sudo snap remove <pkg> --revision <rev>".to_string())
                }
            }
            PurgeAction::Command(cmd, args) if cmd == "apt-get" || cmd == "apt" => {
                Some(format!("sudo {} {}", cmd, args.join(" ")))
            }
            PurgeAction::JournalVacuum(size) => {
                Some(format!("sudo journalctl --vacuum-size={}", size))
            }
            _ => None,
        }
    }
}

fn run_sudo_command(cmd: &str, args: &[&str], password: &str) -> Result<(), String> {
    use std::io::Write;
    use std::process::Stdio;

    let mut child = Command::new("sudo")
        .arg("-S")
        .arg(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn sudo {}: {}", cmd, e))?;

    if let Some(mut stdin) = child.stdin.take() {
        let trimmed_pass = password
            .strip_suffix('\n')
            .unwrap_or(password)
            .strip_suffix('\r')
            .unwrap_or(password);
        let mut pass_buf = Vec::with_capacity(trimmed_pass.len() + 1);
        pass_buf.extend_from_slice(trimmed_pass.as_bytes());
        pass_buf.push(b'\n');

        let write_res = stdin.write_all(&pass_buf);
        let flush_res = stdin.flush();

        // Zeroize password buffer immediately after writing
        for byte in pass_buf.iter_mut() {
            *byte = 0;
        }
        pass_buf.clear();

        // Close stdin so sudo does not wait indefinitely
        drop(stdin);

        if let Err(e) = write_res.and(flush_res) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!("Failed to write password to sudo stdin: {}", e));
        }
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("Failed to wait for sudo {}: {}", cmd, e))?;

    if output.status.success() {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        let trimmed = err.trim();
        if trimmed.is_empty() {
            Err(format!("sudo {} exited with status {}", cmd, output.status))
        } else {
            Err(format!("sudo {} error: {}", cmd, trimmed))
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
            description: format!(
                "{} old disabled revisions (Manual: sudo snap remove <pkg> --revision <rev>)",
                count
            ),
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
    // 1b. NPX Ephemeral Cache
    if let Some(npx) = probe_npx_cache_in(&home.join(".npm/_npx")) {
        targets.push(npx);
    }


    // 2. Python Pip Cache
    let pip_cache = home.join(".cache/pip");
    if pip_cache.is_dir() {
        let size = calculate_dir_size(&pip_cache);
        if size > 1024 * 1024 {
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
                description:
                    "Cached .deb installer archives in /var/cache/apt (Manual: sudo apt-get clean)"
                        .into(),
                purge_action: PurgeAction::Command("apt-get".into(), vec!["clean".into()]),
                requires_sudo: true,
            });
        }
    }
    None
}

/// Probe uv Python runtimes and tool environments in ~/.local/share/uv
pub fn probe_uv_data() -> Option<CleanableTarget> {
    let home = match std::env::var("HOME") {
        Ok(h) => PathBuf::from(h),
        Err(_) => return None,
    };
    let uv_dir = home.join(".local/share/uv");
    if uv_dir.is_dir() {
        let size = calculate_dir_size(&uv_dir);
        if size > 20 * 1024 * 1024 {
            return Some(CleanableTarget {
                id: "uv-data".into(),
                name: "Python uv Tool & Python Runtimes".into(),
                size,
                description: "Cached tools and Python standalone builds in ~/.local/share/uv"
                    .into(),
                purge_action: PurgeAction::DirectoryDelete(uv_dir),
                requires_sudo: false,
            });
        }
    }
    None
}

/// Probe user-owned temporary files in /tmp exceeding 50 MB
pub fn probe_tmp_directory() -> Option<CleanableTarget> {
    let tmp_path = Path::new("/tmp");
    if !tmp_path.is_dir() {
        return None;
    }

    #[cfg(unix)]
    let my_uid = {
        use std::os::unix::fs::MetadataExt;
        if let Ok(home) = std::env::var("HOME") {
            std::fs::metadata(home).map(|m| m.uid()).ok()
        } else {
            None
        }
    };

    let mut user_paths = Vec::new();
    let mut total_size = 0u64;

    if let Ok(entries) = std::fs::read_dir(tmp_path) {
        for entry in entries.flatten() {
            let path = entry.path();

            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                if let Some(uid) = my_uid {
                    if let Ok(meta) = entry.metadata() {
                        if meta.uid() != uid {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }
            }

            let file_name = entry.file_name();
            let name_str = file_name.to_string_lossy();
            if name_str.starts_with(".X11-unix")
                || name_str.starts_with(".ICE-unix")
                || name_str.starts_with(".Test-unix")
            {
                continue;
            }

            let size = if path.is_dir() {
                calculate_dir_size(&path)
            } else {
                entry.metadata().map(|m| m.len()).unwrap_or(0)
            };

            if size > 0 {
                total_size = total_size.saturating_add(size);
                user_paths.push(path);
            }
        }
    }

    if total_size > 50 * 1024 * 1024 {
        Some(CleanableTarget {
            id: "tmp".into(),
            name: "User Temporary Debris (/tmp)".into(),
            size: total_size,
            description: "User-owned temporary files and build artifacts in /tmp".into(),
            purge_action: PurgeAction::TmpUserDebris(user_paths),
            requires_sudo: false,
        })
    } else {
        None
    }
}

/// Probe Snap obsolete user profiles in ~/snap/<app>/<old_rev>
pub fn probe_snap_user_profiles() -> Option<CleanableTarget> {
    let home = std::env::var("HOME").ok().map(PathBuf::from)?;
    probe_snap_user_profiles_in(&home.join("snap"))
}

pub(crate) fn probe_snap_user_profiles_in(snap_dir: &Path) -> Option<CleanableTarget> {
    if !snap_dir.is_dir() {
        return None;
    }

    let mut obsolete_dirs = Vec::new();
    let mut total_size = 0u64;

    if let Ok(entries) = std::fs::read_dir(snap_dir) {
        for app_entry in entries.flatten() {
            let app_path = app_entry.path();
            if !app_path.is_dir() {
                continue;
            }

            let current_target_name = std::fs::read_link(app_path.join("current"))
                .ok()
                .and_then(|p| p.file_name().map(|f| f.to_string_lossy().to_string()));

            if let Ok(rev_entries) = std::fs::read_dir(&app_path) {
                for rev_entry in rev_entries.flatten() {
                    let file_type = match rev_entry.file_type() {
                        Ok(ft) => ft,
                        Err(_) => continue,
                    };
                    if !file_type.is_dir() || file_type.is_symlink() {
                        continue;
                    }

                    let dir_name = rev_entry.file_name().to_string_lossy().to_string();
                    if dir_name.is_empty() || !dir_name.chars().all(|c| c.is_ascii_digit()) {
                        continue;
                    }

                    if Some(&dir_name) == current_target_name.as_ref() {
                        continue;
                    }

                    let p = rev_entry.path();
                    let size = calculate_dir_size(&p);
                    if size > 0 {
                        total_size = total_size.saturating_add(size);
                        obsolete_dirs.push(p);
                    }
                }
            }
        }
    }

    if !obsolete_dirs.is_empty() && total_size > 0 {
        let count = obsolete_dirs.len();
        Some(CleanableTarget {
            id: "snap-profiles".into(),
            name: "Snap Obsolete User Profiles".into(),
            size: total_size,
            description: format!("{} obsolete user profiles across ~/snap packages", count),
            purge_action: PurgeAction::MultiDirectoryDelete(obsolete_dirs),
            requires_sudo: false,
        })
    } else {
        None
    }
}

/// Probe Systemd journal logs exceeding 100MB
pub fn probe_system_journal() -> Option<CleanableTarget> {
    let mut size = 0u64;

    if let Ok(output) = Command::new("journalctl").arg("--disk-usage").output() {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            for word in text.split_whitespace() {
                let parsed = parse_size_string(word);
                if parsed > 0 {
                    size = parsed;
                    break;
                }
            }
        }
    }

    if size == 0 {
        let journal_dir = Path::new("/var/log/journal");
        if journal_dir.is_dir() {
            size = calculate_dir_size(journal_dir);
        }
    }

    if size > 100 * 1024 * 1024 {
        Some(CleanableTarget {
            id: "journal".into(),
            name: "Systemd Journal Logs".into(),
            size,
            description: "Archived systemd journal logs exceeding 100MB".into(),
            purge_action: PurgeAction::JournalVacuum("100M".into()),
            requires_sudo: true,
        })
    } else {
        None
    }
}

/// Probe accumulated Cockpit tools update archives in ~/.local/share/cockpit-tools/updates
pub fn probe_cockpit_updates() -> Option<CleanableTarget> {
    let home = std::env::var("HOME").ok().map(PathBuf::from)?;
    probe_cockpit_updates_in(&home.join(".local/share/cockpit-tools/updates"))
}

pub(crate) fn probe_cockpit_updates_in(updates_dir: &Path) -> Option<CleanableTarget> {
    if !updates_dir.is_dir() {
        return None;
    }

    let total_dir_size = calculate_dir_size(updates_dir);
    if total_dir_size <= 50 * 1024 * 1024 {
        return None;
    }

    let mut version_dirs = Vec::new();
    if let Ok(entries) = std::fs::read_dir(updates_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let modified = entry
                    .metadata()
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                let name = entry.file_name();
                version_dirs.push((modified, name, path));
            }
        }
    }

    version_dirs.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.cmp(&a.1)));

    if version_dirs.len() > 1 {
        let old_dirs: Vec<PathBuf> = version_dirs.into_iter().skip(1).map(|(_, _, p)| p).collect();
        let reclaimable_size: u64 = old_dirs.iter().map(|p| calculate_dir_size(p)).sum();
        if reclaimable_size > 0 {
            let count = old_dirs.len();
            return Some(CleanableTarget {
                id: "cockpit-updates".into(),
                name: "Cockpit Tools Old Updates".into(),
                size: reclaimable_size,
                description: format!(
                    "{} old update versions in ~/.local/share/cockpit-tools/updates",
                    count
                ),
                purge_action: PurgeAction::MultiDirectoryDelete(old_dirs),
                requires_sudo: false,
            });
        }
    }

    None
}

/// Probe Codex runtime dependencies cache in ~/.cache/codex-runtimes
pub fn probe_codex_runtimes() -> Option<CleanableTarget> {
    let home = std::env::var("HOME").ok().map(PathBuf::from)?;
    probe_codex_runtimes_in(&home.join(".cache/codex-runtimes"))
}

pub(crate) fn probe_codex_runtimes_in(dir: &Path) -> Option<CleanableTarget> {
    if dir.is_dir() {
        let size = calculate_dir_size(dir);
        if size > 50 * 1024 * 1024 {
            return Some(CleanableTarget {
                id: "codex-runtimes".into(),
                name: "Codex Runtime Dependencies Cache".into(),
                size,
                description: "Cached runtime dependencies in ~/.cache/codex-runtimes".into(),
                purge_action: PurgeAction::DirectoryDelete(dir.to_path_buf()),
                requires_sudo: false,
            });
        }
    }
    None
}

/// Probe Gradle wrapper distribution downloads in ~/.gradle/wrapper/dists
pub fn probe_gradle_wrappers() -> Option<CleanableTarget> {
    let home = std::env::var("HOME").ok().map(PathBuf::from)?;
    probe_gradle_wrappers_in(&home.join(".gradle/wrapper/dists"))
}

pub(crate) fn probe_gradle_wrappers_in(dir: &Path) -> Option<CleanableTarget> {
    if dir.is_dir() {
        let size = calculate_dir_size(dir);
        if size > 50 * 1024 * 1024 {
            return Some(CleanableTarget {
                id: "gradle-wrapper".into(),
                name: "Gradle Wrapper Distributions".into(),
                size,
                description:
                    "Cached Gradle wrapper distribution zips and unpacks in ~/.gradle/wrapper/dists"
                        .into(),
                purge_action: PurgeAction::DirectoryDelete(dir.to_path_buf()),
                requires_sudo: false,
            });
        }
    }
    None
}
/// Probe NPX ephemeral execution cache in ~/.npm/_npx
pub fn probe_npx_cache() -> Option<CleanableTarget> {
    let home = std::env::var("HOME").ok().map(PathBuf::from)?;
    probe_npx_cache_in(&home.join(".npm/_npx"))
}

pub(crate) fn probe_npx_cache_in(dir: &Path) -> Option<CleanableTarget> {
    if !dir.is_dir() {
        return None;
    }
    let size = calculate_dir_size(dir);
    if size > 10 * 1024 * 1024 {
        Some(CleanableTarget {
            id: "npx".into(),
            name: "NPX Ephemeral Cache".into(),
            size,
            description: "Cached temporary binaries and packages in ~/.npm/_npx".into(),
            purge_action: PurgeAction::DirectoryDelete(dir.to_path_buf()),
            requires_sudo: false,
        })
    } else {
        None
    }
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
    if let Some(snap_profiles) = probe_snap_user_profiles() {
        targets.push(snap_profiles);
    }
    if let Some(journal) = probe_system_journal() {
        targets.push(journal);
    }
    targets.extend(probe_package_caches());
    if let Some(cockpit) = probe_cockpit_updates() {
        targets.push(cockpit);
    }
    if let Some(codex) = probe_codex_runtimes() {
        targets.push(codex);
    }
    if let Some(gradle) = probe_gradle_wrappers() {
        targets.push(gradle);
    }
    if let Some(uv) = probe_uv_data() {
        targets.push(uv);
    }
    if let Some(tmp) = probe_tmp_directory() {
        targets.push(tmp);
    }
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

    #[test]
    fn test_purge_action_requires_sudo_and_guidance() {
        let snap = PurgeAction::SnapRevisions(vec![("discord".into(), "123".into())]);
        assert!(snap.requires_sudo());
        assert_eq!(
            snap.manual_guidance(),
            Some("sudo snap remove discord --revision 123".to_string())
        );

        let apt = PurgeAction::Command("apt-get".into(), vec!["clean".into()]);
        assert!(apt.requires_sudo());
        assert_eq!(
            apt.manual_guidance(),
            Some("sudo apt-get clean".to_string())
        );

        let journal = PurgeAction::JournalVacuum("100M".into());
        assert!(journal.requires_sudo());
        assert_eq!(
            journal.manual_guidance(),
            Some("sudo journalctl --vacuum-size=100M".to_string())
        );

        let dir = PurgeAction::DirectoryDelete(PathBuf::from("/tmp"));
        assert!(!dir.requires_sudo());
        assert_eq!(dir.manual_guidance(), None);

        let multi = PurgeAction::MultiDirectoryDelete(vec![PathBuf::from("/tmp/a")]);
        assert!(!multi.requires_sudo());
        assert_eq!(multi.manual_guidance(), None);

        let tmp_debris = PurgeAction::TmpUserDebris(vec![]);
        assert!(!tmp_debris.requires_sudo());
        assert_eq!(tmp_debris.manual_guidance(), None);
    }

    #[test]
    fn test_purge_action_tmp_user_debris() {
        let temp_dir = std::env::temp_dir().join("test_tmp_user_debris_dir");
        let _ = std::fs::create_dir_all(&temp_dir);
        let sample_file = temp_dir.join("subfile.txt");
        let _ = std::fs::write(&sample_file, b"sample");
        assert!(sample_file.exists());

        let action = PurgeAction::TmpUserDebris(vec![temp_dir.clone()]);
        assert!(action.execute().is_ok());
        assert!(!sample_file.exists());
        assert!(!temp_dir.exists());
    }

    #[test]
    fn test_purge_action_multi_directory_delete() {
        let base = std::env::temp_dir().join("test_multi_dir_del");
        let dir1 = base.join("dir1");
        let dir2 = base.join("dir2");
        let _ = std::fs::create_dir_all(&dir1);
        let _ = std::fs::create_dir_all(&dir2);
        let _ = std::fs::write(dir1.join("a.txt"), b"a");
        let _ = std::fs::write(dir2.join("b.txt"), b"b");
        assert!(dir1.exists());
        assert!(dir2.exists());

        let action = PurgeAction::MultiDirectoryDelete(vec![dir1.clone(), dir2.clone()]);
        let res = action.execute();
        assert!(res.is_ok());
        assert!(!dir1.exists());
        assert!(!dir2.exists());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn test_execute_with_sudo_delegates_for_non_sudo() {
        let temp_dir = std::env::temp_dir().join("test_exec_sudo_del");
        let _ = std::fs::create_dir_all(&temp_dir);
        let f = temp_dir.join("f.txt");
        let _ = std::fs::write(&f, b"test");
        assert!(f.exists());

        let action = PurgeAction::DirectoryDelete(temp_dir.clone());
        let res = action.execute_with_sudo("fake_password");
        assert!(res.is_ok());
        assert!(!f.exists());
        let _ = std::fs::remove_dir_all(&temp_dir);

        let snap_empty = PurgeAction::SnapRevisions(vec![]);
        assert!(snap_empty.execute_with_sudo("fake_password").is_ok());
    }

    #[test]
    fn test_probe_snap_user_profiles() {
        let temp_base = std::env::temp_dir().join("test_snap_profiles");
        let app_dir = temp_base.join("testapp");
        let rev1 = app_dir.join("101");
        let rev2 = app_dir.join("102");
        let common = app_dir.join("common");
        let _ = std::fs::create_dir_all(&rev1);
        let _ = std::fs::create_dir_all(&rev2);
        let _ = std::fs::create_dir_all(&common);

        let _ = std::fs::write(rev1.join("data.txt"), vec![1u8; 1024]);
        let _ = std::fs::write(rev2.join("data.txt"), vec![1u8; 1024]);

        #[cfg(unix)]
        let _ = std::os::unix::fs::symlink("102", app_dir.join("current"));

        let target = probe_snap_user_profiles_in(&temp_base);
        #[cfg(unix)]
        {
            assert!(target.is_some());
            let t = target.unwrap();
            assert_eq!(t.id, "snap-profiles");
            assert_eq!(t.name, "Snap Obsolete User Profiles");
            assert!(!t.requires_sudo);
            match t.purge_action {
                PurgeAction::MultiDirectoryDelete(dirs) => {
                    assert_eq!(dirs.len(), 1);
                    assert_eq!(dirs[0], rev1);
                }
                _ => panic!("Expected MultiDirectoryDelete"),
            }
        }
        let _ = std::fs::remove_dir_all(&temp_base);
    }

    #[test]
    fn test_probe_cockpit_updates() {
        let temp_base = std::env::temp_dir().join("test_cockpit_updates");
        let v1 = temp_base.join("v1.0.0");
        let v2 = temp_base.join("v2.0.0");
        let _ = std::fs::create_dir_all(&v1);
        let _ = std::fs::create_dir_all(&v2);

        // Create files exceeding 50MB in total
        let f1 = std::fs::File::create(v1.join("pkg.tar")).unwrap();
        f1.set_len(30 * 1024 * 1024).unwrap();
        // Brief sleep to ensure distinct modified time
        std::thread::sleep(std::time::Duration::from_millis(15));
        let f2 = std::fs::File::create(v2.join("pkg.tar")).unwrap();
        f2.set_len(30 * 1024 * 1024).unwrap();

        let target = probe_cockpit_updates_in(&temp_base);
        assert!(target.is_some());
        let t = target.unwrap();
        assert_eq!(t.id, "cockpit-updates");
        assert_eq!(t.name, "Cockpit Tools Old Updates");
        assert!(!t.requires_sudo);
        match t.purge_action {
            PurgeAction::MultiDirectoryDelete(dirs) => {
                assert_eq!(dirs.len(), 1);
                assert_eq!(dirs[0], v1);
            }
            _ => panic!("Expected MultiDirectoryDelete"),
        }
        let _ = std::fs::remove_dir_all(&temp_base);
    }

    #[test]
    fn test_probe_codex_runtimes() {
        let temp_dir = std::env::temp_dir().join("test_codex_runtimes");
        let _ = std::fs::create_dir_all(&temp_dir);
        let f = std::fs::File::create(temp_dir.join("runtime.bin")).unwrap();
        f.set_len(55 * 1024 * 1024).unwrap();

        let target = probe_codex_runtimes_in(&temp_dir);
        assert!(target.is_some());
        let t = target.unwrap();
        assert_eq!(t.id, "codex-runtimes");
        assert_eq!(t.name, "Codex Runtime Dependencies Cache");
        assert!(!t.requires_sudo);
        match t.purge_action {
            PurgeAction::DirectoryDelete(d) => assert_eq!(d, temp_dir),
            _ => panic!("Expected DirectoryDelete"),
        }
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_probe_gradle_wrappers() {
        let temp_dir = std::env::temp_dir().join("test_gradle_wrappers");
        let _ = std::fs::create_dir_all(&temp_dir);
        let f = std::fs::File::create(temp_dir.join("gradle-8.0-bin.zip")).unwrap();
        f.set_len(55 * 1024 * 1024).unwrap();

        let target = probe_gradle_wrappers_in(&temp_dir);
        assert!(target.is_some());
        let t = target.unwrap();
        assert_eq!(t.id, "gradle-wrapper");
        assert_eq!(t.name, "Gradle Wrapper Distributions");
        assert!(!t.requires_sudo);
        match t.purge_action {
            PurgeAction::DirectoryDelete(d) => assert_eq!(d, temp_dir),
            _ => panic!("Expected DirectoryDelete"),
        }
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_probe_system_journal() {
        // probe_system_journal should not panic and produce valid fields if present
        if let Some(target) = probe_system_journal() {
            assert_eq!(target.id, "journal");
            assert_eq!(target.name, "Systemd Journal Logs");
            assert!(target.requires_sudo);
            assert!(target.size > 100 * 1024 * 1024);
            match &target.purge_action {
                PurgeAction::JournalVacuum(size) => assert_eq!(size, "100M"),
                _ => panic!("Expected JournalVacuum"),
            }
        }
    }

    #[test]
    fn test_probe_npx_cache() {
        let temp_dir = std::env::temp_dir().join("test_npx_cache_dir");
        let _ = std::fs::remove_dir_all(&temp_dir);

        // Missing directory -> None
        assert!(probe_npx_cache_in(&temp_dir).is_none());

        let _ = std::fs::create_dir_all(&temp_dir);

        // Sub-10MB (5MB) -> None
        let small_file = temp_dir.join("small_pkg.bin");
        let f1 = std::fs::File::create(&small_file).unwrap();
        f1.set_len(5 * 1024 * 1024).unwrap();
        drop(f1);
        assert!(probe_npx_cache_in(&temp_dir).is_none());

        // Exceeding 10MB (add 6MB -> total 11MB) -> Some(target)
        let f2 = std::fs::File::create(temp_dir.join("large_pkg.bin")).unwrap();
        f2.set_len(6 * 1024 * 1024).unwrap();
        drop(f2);

        let target = probe_npx_cache_in(&temp_dir);
        assert!(target.is_some());
        let t = target.unwrap();
        assert_eq!(t.id, "npx");
        assert_eq!(t.name, "NPX Ephemeral Cache");
        assert!(!t.requires_sudo);
        assert_eq!(t.size, 11 * 1024 * 1024);
        match &t.purge_action {
            PurgeAction::DirectoryDelete(p) => assert_eq!(p, &temp_dir),
            _ => panic!("Expected DirectoryDelete"),
        }

        // PurgeAction safely empties contents and re-creates empty dir
        let purge_res = t.purge_action.execute();
        assert!(purge_res.is_ok());
        assert!(temp_dir.is_dir());
        assert_eq!(calculate_dir_size(&temp_dir), 0);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_probes_extended_resilience() {
        let _ = probe_uv_data();
        let _ = probe_tmp_directory();
        let _ = probe_snap_user_profiles();
        let _ = probe_system_journal();
        let _ = probe_cockpit_updates();
        let _ = probe_codex_runtimes();
        let _ = probe_gradle_wrappers();
        let _ = probe_npx_cache();
    }
}
