//! Map images: the levelshots inside a game's own `.pk3` files (Quake 3
//! engine games), found on the user's disk. Nothing is downloaded and no
//! image ships with OpenASE (ADR 0004).

use std::io::Read;
use std::path::{Path, PathBuf};

const EXTENSIONS: [&str; 3] = ["jpg", "tga", "png"];
/// Larger entries are not images we want to decode.
const MAX_IMAGE: u64 = 8 * 1024 * 1024;

/// A map name from a server, safe to use in a path inside an archive:
/// `/`-separated segments, none empty, `.` or `..`, no `\` or `:`.
pub fn valid_map_name(map: &str) -> bool {
    !map.is_empty()
        && !map.contains(['\\', ':'])
        && map
            .split('/')
            .all(|seg| !seg.is_empty() && seg != "." && seg != "..")
}

/// `.pk3` files of a folder in reverse load order: the engine loads them
/// alphabetically and a later one overrides an earlier one.
fn archives(dir: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("pk3"))
        })
        .collect();
    found.sort_by_key(|path| path.file_name().map(|n| n.to_string_lossy().to_lowercase()));
    found.reverse();
    found
}

fn read_entry(archive: &Path, wanted: &[String]) -> Option<Vec<u8>> {
    let mut zip = zip::ZipArchive::new(std::fs::File::open(archive).ok()?).ok()?;
    let index = (0..zip.len()).find(|&i| {
        zip.name_for_index(i)
            .and_then(Result::ok)
            .is_some_and(|name| wanted.contains(&name.to_lowercase()))
    })?;
    let entry = zip.by_index(index).ok()?;
    if entry.size() > MAX_IMAGE {
        return None;
    }
    let mut data = Vec::new();
    entry.take(MAX_IMAGE).read_to_end(&mut data).ok()?;
    Some(data)
}

/// Bytes of `levelshots/<map>.{jpg,tga,png}` from the `.pk3` files of
/// `dirs`, in order (the server's mod first, then the base game). Damaged
/// archives are skipped.
pub fn find_levelshot(dirs: &[PathBuf], map: &str) -> Option<Vec<u8>> {
    if !valid_map_name(map) {
        return None;
    }
    let wanted: Vec<String> = EXTENSIONS
        .iter()
        .map(|ext| format!("levelshots/{map}.{ext}").to_lowercase())
        .collect();
    dirs.iter()
        .flat_map(|dir| archives(dir))
        .find_map(|archive| read_entry(&archive, &wanted))
}

/// Decode an image to `([width, height], RGBA bytes)`.
pub fn decode(bytes: &[u8]) -> Option<([usize; 2], Vec<u8>)> {
    // TGA has no magic bytes to guess from: tried when nothing else matches.
    let image = image::load_from_memory(bytes)
        .or_else(|_| image::load_from_memory_with_format(bytes, image::ImageFormat::Tga))
        .ok()?
        .into_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    Some((size, image.into_raw()))
}
