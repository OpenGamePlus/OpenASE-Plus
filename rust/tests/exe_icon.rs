//! Icons from Windows executables: a minimal PE with a resource section is
//! built here (DOS stub, PE32+ headers, one `.rsrc` section holding an
//! RT_GROUP_ICON and its RT_ICON images).

use openase::exe_icon::{decode_icon, icon_from_pe};

const RSRC_RVA: u32 = 0x1000;
const RSRC_FILE: u32 = 0x200;

fn png(side: u32, rgb: [u8; 3]) -> Vec<u8> {
    let image =
        image::RgbaImage::from_pixel(side, side, image::Rgba([rgb[0], rgb[1], rgb[2], 255]));
    let mut out = std::io::Cursor::new(Vec::new());
    image.write_to(&mut out, image::ImageFormat::Png).unwrap();
    out.into_inner()
}

fn put(buf: &mut Vec<u8>, at: usize, bytes: &[u8]) {
    if buf.len() < at + bytes.len() {
        buf.resize(at + bytes.len(), 0);
    }
    buf[at..at + bytes.len()].copy_from_slice(bytes);
}

/// A resource directory with `entries` (id, offset with the subdirectory bit).
fn directory(entries: &[(u32, u32)]) -> Vec<u8> {
    let mut out = vec![0u8; 16];
    out[14..16].copy_from_slice(&(entries.len() as u16).to_le_bytes());
    for (id, offset) in entries {
        out.extend_from_slice(&id.to_le_bytes());
        out.extend_from_slice(&offset.to_le_bytes());
    }
    out
}

/// A PE with icons of the given sides (each its own RT_ICON id 1, 2...).
fn pe(icons: &[(u32, Vec<u8>)]) -> Vec<u8> {
    // .rsrc layout (offsets inside the section):
    //   0: root -> type 3 (icons) and type 14 (group)
    //   then per type: name directory -> language directory -> data entry
    let n = icons.len() as u32;
    let mut rsrc = Vec::new();
    let sub = 0x8000_0000u32;
    // Fixed slots: root 0x000, icon types 0x100.., group 0x400.., data 0x800..
    put(
        &mut rsrc,
        0x000,
        &directory(&[(3, sub | 0x100), (14, sub | 0x200)]),
    );
    let icon_names: Vec<(u32, u32)> = (0..n).map(|i| (i + 1, sub | (0x300 + i * 0x20))).collect();
    put(&mut rsrc, 0x100, &directory(&icon_names));
    put(&mut rsrc, 0x200, &directory(&[(1, sub | 0x280)]));
    put(&mut rsrc, 0x280, &directory(&[(1033, 0x700)])); // group language -> data entry
    for i in 0..n {
        put(
            &mut rsrc,
            (0x300 + i * 0x20) as usize,
            &directory(&[(1033, 0x710 + i * 0x10)]),
        );
    }
    // Group data: GRPICONDIR + 14-byte entries.
    let mut group = vec![0, 0, 1, 0];
    group.extend_from_slice(&(n as u16).to_le_bytes());
    for (i, (side, data)) in icons.iter().enumerate() {
        let w = if *side >= 256 { 0 } else { *side as u8 };
        group.extend_from_slice(&[w, w, 0, 0, 1, 0, 32, 0]);
        group.extend_from_slice(&(data.len() as u32).to_le_bytes());
        group.extend_from_slice(&((i + 1) as u16).to_le_bytes());
    }
    let mut at = 0x800u32;
    let mut entry = |rsrc: &mut Vec<u8>, slot: u32, data: &[u8]| {
        put(rsrc, slot as usize, &(RSRC_RVA + at).to_le_bytes());
        put(rsrc, slot as usize + 4, &(data.len() as u32).to_le_bytes());
        put(rsrc, at as usize, data);
        at += (data.len() as u32 + 15) & !15;
    };
    entry(&mut rsrc, 0x700, &group);
    for (i, (_, data)) in icons.iter().enumerate() {
        entry(&mut rsrc, 0x710 + i as u32 * 0x10, data);
    }

    let mut file = vec![0u8; RSRC_FILE as usize];
    file[0..2].copy_from_slice(b"MZ");
    file[0x3c..0x40].copy_from_slice(&0x40u32.to_le_bytes());
    let pe = 0x40usize;
    file[pe..pe + 4].copy_from_slice(b"PE\0\0");
    file[pe + 6..pe + 8].copy_from_slice(&1u16.to_le_bytes()); // one section
    let opt_size = 112 + 16 * 8;
    file[pe + 20..pe + 22].copy_from_slice(&(opt_size as u16).to_le_bytes());
    let opt = pe + 24;
    file[opt..opt + 2].copy_from_slice(&0x20bu16.to_le_bytes()); // PE32+
    let dirs = opt + 112;
    file[dirs + 16..dirs + 20].copy_from_slice(&RSRC_RVA.to_le_bytes()); // resource dir
    file[dirs + 20..dirs + 24].copy_from_slice(&(rsrc.len() as u32).to_le_bytes());
    let section = opt + opt_size;
    file[section..section + 8].copy_from_slice(b".rsrc\0\0\0");
    file[section + 8..section + 12].copy_from_slice(&(rsrc.len() as u32).to_le_bytes());
    file[section + 12..section + 16].copy_from_slice(&RSRC_RVA.to_le_bytes());
    file[section + 16..section + 20].copy_from_slice(&(rsrc.len() as u32).to_le_bytes());
    file[section + 20..section + 24].copy_from_slice(&RSRC_FILE.to_le_bytes());
    file.extend_from_slice(&rsrc);
    file
}

#[test]
fn the_32_pixel_icon_is_preferred_and_decodes() {
    let exe = pe(&[
        (16, png(16, [255, 0, 0])),
        (32, png(32, [0, 0, 255])),
        (256, png(256, [0, 255, 0])),
    ]);
    let ico = icon_from_pe(&exe).unwrap();
    let (size, rgba) = decode_icon(&ico).unwrap();
    assert_eq!(size, [32, 32]);
    assert_eq!(&rgba[..4], &[0, 0, 255, 255]);
}

#[test]
fn a_lone_large_icon_is_used() {
    let exe = pe(&[(256, png(256, [0, 255, 0]))]);
    let (size, _) = decode_icon(&icon_from_pe(&exe).unwrap()).unwrap();
    assert_eq!(size, [256, 256]);
}

#[test]
fn non_pe_and_truncated_files_have_no_icon() {
    assert!(icon_from_pe(b"\x7fELF not windows").is_none());
    assert!(icon_from_pe(b"MZ").is_none());
    let exe = pe(&[(32, png(32, [1, 2, 3]))]);
    assert!(icon_from_pe(&exe[..exe.len() - 40]).is_none());
    assert!(icon_from_pe(&exe[..0x150]).is_none());
}
