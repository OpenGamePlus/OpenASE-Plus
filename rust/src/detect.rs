//! Finding installed games: Windows registry first, then a disk search
//! (ASE's "Auto-Search").

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::catalog::Game;

/// Return the first executable found through the game's registry entries.
pub fn detect_from_registry(game: &Game) -> Option<PathBuf> {
    game.registry.iter().find_map(|entry| {
        let folder = registry_value(&entry.key, &entry.value)?;
        let candidate = PathBuf::from(folder.trim().trim_matches('"')).join(&entry.exe);
        candidate.is_file().then_some(candidate)
    })
}

#[cfg(windows)]
fn registry_value(key: &str, value: &str) -> Option<String> {
    use winreg::RegKey;
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};

    let (root_name, subkey) = key.split_once('\\')?;
    let root = match root_name {
        "HKEY_LOCAL_MACHINE" => RegKey::predef(HKEY_LOCAL_MACHINE),
        "HKEY_CURRENT_USER" => RegKey::predef(HKEY_CURRENT_USER),
        _ => return None,
    };
    // 32-bit games on 64-bit Windows register under WOW6432Node.
    let wow = subkey
        .strip_prefix("SOFTWARE\\")
        .or_else(|| subkey.strip_prefix("Software\\"))
        .map(|rest| format!("SOFTWARE\\WOW6432Node\\{rest}"));
    std::iter::once(subkey.to_string())
        .chain(wow)
        .find_map(|path| {
            root.open_subkey(path)
                .ok()?
                .get_value::<String, _>(value)
                .ok()
        })
}

#[cfg(not(windows))]
fn registry_value(_key: &str, _value: &str) -> Option<String> {
    None
}

/// Folders searched by Auto-Search, most likely first.
pub fn search_roots() -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = ["ProgramFiles(x86)", "ProgramFiles", "ProgramW6432"]
        .iter()
        .filter_map(std::env::var_os)
        .map(PathBuf::from)
        .collect();
    if cfg!(windows) {
        for letter in b'C'..=b'Z' {
            let drive = PathBuf::from(format!("{}:\\", letter as char));
            if drive.is_dir() {
                roots.push(drive);
            }
        }
    } else {
        let home = std::env::var_os("HOME").map(PathBuf::from);
        if let Some(home) = &home {
            // Steam libraries first: that is where most games live.
            for steam in [
                ".steam/steam/steamapps/common",
                ".local/share/Steam/steamapps/common",
                ".var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/common",
                "Library/Application Support/Steam/steamapps/common",
            ] {
                let dir = home.join(steam);
                if dir.is_dir() {
                    roots.push(dir);
                }
            }
        }
        for system in ["/Applications", "/usr/games", "/usr/local/games", "/opt"] {
            let dir = PathBuf::from(system);
            if dir.is_dir() {
                roots.push(dir);
            }
        }
        if let Some(home) = home {
            roots.push(home);
        }
    }
    let mut seen = std::collections::HashSet::new();
    roots.retain(|root| seen.insert(root.to_string_lossy().to_lowercase()));
    roots
}

/// Search `roots` for any of `exe_names`, reporting each folder visited.
/// Stops early when `cancel` is set. Returns every match found.
pub fn search(
    roots: &[PathBuf],
    exe_names: &[String],
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(&Path),
    found: &mut dyn FnMut(&Path),
) {
    let mut visited = std::collections::HashSet::new();
    for root in roots {
        walk(root, exe_names, cancel, progress, found, &mut visited);
    }
}

fn walk(
    dir: &Path,
    exe_names: &[String],
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(&Path),
    found: &mut dyn FnMut(&Path),
    visited: &mut std::collections::HashSet<String>,
) {
    if cancel.load(Ordering::Relaxed) || !visited.insert(dir.to_string_lossy().to_lowercase()) {
        return;
    }
    let skip = dir
        .file_name()
        .map(|name| name.to_string_lossy().to_lowercase())
        .is_some_and(|name| {
            matches!(
                name.as_str(),
                "windows"
                    | "$recycle.bin"
                    | "system volume information"
                    | "winsxs"
                    | ".cache"
                    | ".git"
                    | "node_modules"
                    | ".trash"
                    | "proc"
            )
        });
    if skip {
        return;
    }
    progress(dir);
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut subdirs = Vec::new();
    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_dir() {
            subdirs.push(entry.path());
        } else if file_type.is_file() {
            let name = entry.file_name().to_string_lossy().to_string();
            if exe_names.iter().any(|exe| names_match(&name, exe)) {
                found(&entry.path());
            }
        }
    }
    for subdir in subdirs {
        walk(&subdir, exe_names, cancel, progress, found, visited);
    }
}

/// Games an executable may belong to, each with whether it is sure.
pub type Owners = Vec<(&'static crate::catalog::Game, bool)>;

/// The games an executable found on disk can belong to, each with whether
/// it is sure: the file name is only that game's, or the game's install
/// folder (from its registry entry) is in the path. `hl2.exe` in
/// `.../Counter-Strike Source/` is surely CS:Source; elsewhere it is only
/// a guess, left for the user. Names compare without case (Windows, Wine).
pub fn owners(path: &Path) -> Owners {
    let Some(name) = path.file_name().map(|n| n.to_string_lossy().to_lowercase()) else {
        return Vec::new();
    };
    let folders: Vec<String> = path
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_lowercase())
        .collect();
    let candidates: Vec<&crate::catalog::Game> = crate::catalog::games()
        .iter()
        .filter(|game| game.exe_names.iter().any(|exe| exe.to_lowercase() == name))
        .collect();
    let alone = candidates.len() == 1;
    candidates
        .into_iter()
        .map(|game| {
            let in_its_folder = game.registry.iter().any(|entry| {
                let parts: Vec<&str> = entry.exe.split(['\\', '/']).collect();
                parts.len() >= 2 && folders.contains(&parts[parts.len() - 2].to_lowercase())
            });
            (game, alone || in_its_folder)
        })
        .collect()
}

/// One walk of `roots` for the executables of every game; `found` gets
/// each match with its [`owners`].
pub fn search_all(
    roots: &[PathBuf],
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(&Path),
    found: &mut dyn FnMut(&Path, Owners),
) {
    let mut names: Vec<String> = crate::catalog::games()
        .iter()
        .flat_map(|game| game.exe_names.iter().cloned())
        .collect();
    names.sort();
    names.dedup();
    search(roots, &names, cancel, progress, &mut |path| {
        found(path, owners(path))
    });
}

fn names_match(a: &str, b: &str) -> bool {
    if cfg!(windows) {
        a.eq_ignore_ascii_case(b)
    } else {
        a == b
    }
}
