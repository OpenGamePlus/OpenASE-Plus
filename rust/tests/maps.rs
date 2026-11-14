//! Map images from the levelshots in a game's own .pk3 files.

use std::io::Write;
use std::path::Path;

use openase::maps::{decode, find_levelshot, valid_map_name};

fn pk3(path: &Path, files: &[(&str, &[u8])]) {
    let mut zip = zip::ZipWriter::new(std::fs::File::create(path).unwrap());
    let options = zip::write::SimpleFileOptions::default();
    for (name, data) in files {
        zip.start_file(*name, options).unwrap();
        zip.write_all(data).unwrap();
    }
    zip.finish().unwrap();
}

#[test]
fn map_names_from_servers_are_checked() {
    for good in ["q3dm17", "ut4_abbey", "mp/q3ctf1", "DM-Deck16]["] {
        assert!(valid_map_name(good), "{good}");
    }
    for bad in [
        "",
        "../q3dm17",
        "a/../b",
        "a\\b",
        "c:x",
        "a//b",
        "/abs",
        ".",
    ] {
        assert!(!valid_map_name(bad), "{bad}");
    }
}

#[test]
fn the_last_pk3_in_load_order_wins_and_names_ignore_case() {
    let dir = tempfile::tempdir().unwrap();
    pk3(
        &dir.path().join("pak0.pk3"),
        &[("levelshots/q3dm17.jpg", b"old")],
    );
    pk3(
        &dir.path().join("zz-pack.pk3"),
        &[("LevelShots/Q3DM17.TGA", b"new")],
    );
    std::fs::write(dir.path().join("pak9.pk3"), b"not a zip").unwrap();
    pk3(
        &dir.path().join("other.zip"),
        &[("levelshots/q3dm17.jpg", b"ignored")],
    );
    let dirs = [dir.path().to_path_buf()];
    assert_eq!(
        find_levelshot(&dirs, "q3dm17").as_deref(),
        Some(&b"new"[..])
    );
    assert_eq!(find_levelshot(&dirs, "nosuchmap"), None);
    assert_eq!(find_levelshot(&dirs, "../pak0"), None);
    // Earlier folders (the server's mod) win over later ones (the base game).
    let modded = tempfile::tempdir().unwrap();
    pk3(
        &modded.path().join("mod.pk3"),
        &[("levelshots/q3dm17.png", b"mod")],
    );
    let dirs = [modded.path().to_path_buf(), dir.path().to_path_buf()];
    assert_eq!(
        find_levelshot(&dirs, "q3dm17").as_deref(),
        Some(&b"mod"[..])
    );
}

#[test]
fn images_decode_to_rgba() {
    // A 2x1 uncompressed true-color TGA: blue then red pixel (BGR order).
    let mut tga = vec![0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 1, 0, 24, 0x20];
    tga.extend_from_slice(&[255, 0, 0, 0, 0, 255]);
    let (size, rgba) = decode(&tga).unwrap();
    assert_eq!(size, [2, 1]);
    assert_eq!(&rgba[..8], &[0, 0, 255, 255, 255, 0, 0, 255]);
    assert!(decode(b"garbage").is_none());
}
