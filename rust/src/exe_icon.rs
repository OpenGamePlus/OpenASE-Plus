//! The icon of a Windows executable, read from its PE resources (works on
//! any OS, also for games run with Wine). Every read is bounds-checked: a
//! damaged or unusual file gives `None`, never a panic.
//!
//! PE layout used: `MZ` header -> `PE\0\0` -> COFF header -> optional header
//! (PE32 or PE32+) whose data directory 2 is the resource table -> section
//! table to map RVAs to file offsets. Resources are a three-level tree
//! (type, name, language); RT_GROUP_ICON (14) lists the images of an icon,
//! each stored as an RT_ICON (3) resource, from which a one-image `.ico`
//! file is rebuilt.

use std::path::Path;

const RT_ICON: u32 = 3;
const RT_GROUP_ICON: u32 = 14;
/// Executables larger than this are not read.
const MAX_EXE: u64 = 512 * 1024 * 1024;
/// Side preferred for list and tree icons.
const PREFERRED: u32 = 32;

fn u16_at(data: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(data.get(at..at + 2)?.try_into().ok()?))
}

fn u32_at(data: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(data.get(at..at + 4)?.try_into().ok()?))
}

struct Pe<'a> {
    data: &'a [u8],
    /// (virtual address, virtual size, raw size, raw offset) per section.
    sections: Vec<(u32, u32, u32, u32)>,
    /// File offset of the resource directory.
    root: usize,
}

impl<'a> Pe<'a> {
    fn parse(data: &'a [u8]) -> Option<Pe<'a>> {
        if data.get(..2)? != b"MZ" {
            return None;
        }
        let pe = u32_at(data, 0x3c)? as usize;
        if data.get(pe..pe + 4)? != b"PE\0\0" {
            return None;
        }
        let sections = u16_at(data, pe + 6)? as usize;
        let opt_size = u16_at(data, pe + 20)? as usize;
        let opt = pe + 24;
        let dirs = match u16_at(data, opt)? {
            0x10b => opt + 96,
            0x20b => opt + 112,
            _ => return None,
        };
        let resource_rva = u32_at(data, dirs + 2 * 8)?;
        if resource_rva == 0 {
            return None;
        }
        let table = opt + opt_size;
        let sections = (0..sections.min(96))
            .map(|i| {
                let s = table + i * 40;
                Some((
                    u32_at(data, s + 12)?,
                    u32_at(data, s + 8)?,
                    u32_at(data, s + 16)?,
                    u32_at(data, s + 20)?,
                ))
            })
            .collect::<Option<Vec<_>>>()?;
        let mut pe = Pe {
            data,
            sections,
            root: 0,
        };
        pe.root = pe.offset(resource_rva)?;
        Some(pe)
    }

    /// File offset of a relative virtual address.
    fn offset(&self, rva: u32) -> Option<usize> {
        self.sections
            .iter()
            .find_map(|&(va, vsize, raw_size, raw)| {
                let size = vsize.max(raw_size);
                (rva >= va && rva - va < size).then(|| (raw + (rva - va)) as usize)
            })
    }

    /// Entries (id, offset field) of the resource directory at `dir`.
    fn entries(&self, dir: usize) -> Option<Vec<(u32, u32)>> {
        let named = u16_at(self.data, dir + 12)? as usize;
        let ids = u16_at(self.data, dir + 14)? as usize;
        (0..(named + ids).min(4096))
            .map(|i| {
                let at = dir + 16 + i * 8;
                Some((u32_at(self.data, at)?, u32_at(self.data, at + 4)?))
            })
            .collect()
    }

    fn subdir(&self, field: u32) -> Option<usize> {
        (field & 0x8000_0000 != 0).then(|| self.root + (field & 0x7fff_ffff) as usize)
    }

    /// Bytes of the first language of resource `(kind, name)`; `name`
    /// `None` takes the first one.
    fn resource(&self, kind: u32, name: Option<u32>) -> Option<&'a [u8]> {
        let types = self.entries(self.root)?;
        let names = self.subdir(types.iter().find(|(id, _)| *id == kind)?.1)?;
        let names = self.entries(names)?;
        let entry = match name {
            Some(name) => names.iter().find(|(id, _)| *id == name)?,
            None => names.first()?,
        };
        let languages = self.entries(self.subdir(entry.1)?)?;
        let data_entry = self.root + (languages.first()?.1 & 0x7fff_ffff) as usize;
        let rva = u32_at(self.data, data_entry)?;
        let size = u32_at(self.data, data_entry + 4)? as usize;
        let start = self.offset(rva)?;
        self.data.get(start..start.checked_add(size)?)
    }
}

/// A one-image `.ico` file from the main icon of a PE executable: the
/// 32 px image when there is one, else the smallest larger, else the largest.
pub fn icon_from_pe(data: &[u8]) -> Option<Vec<u8>> {
    let pe = Pe::parse(data)?;
    let group = pe.resource(RT_GROUP_ICON, None)?;
    let count = u16_at(group, 4)? as usize;
    // (side in pixels, RT_ICON id, the group entry's first 8 bytes).
    let images: Vec<(u32, u32, &[u8])> = (0..count.min(64))
        .filter_map(|i| {
            let at = 6 + i * 14;
            let side = match *group.get(at)? {
                0 => 256,
                w => u32::from(w),
            };
            let id = u32::from(u16_at(group, at + 12)?);
            (id != 0).then_some((side, id, group.get(at..at + 8)?))
        })
        .collect();
    let (_, id, entry) = images
        .iter()
        .min_by_key(|(side, _, _)| match side.cmp(&PREFERRED) {
            std::cmp::Ordering::Equal => (0, 0),
            std::cmp::Ordering::Greater => (1, *side),
            std::cmp::Ordering::Less => (2, u32::MAX - side),
        })?;
    let image = pe.resource(RT_ICON, Some(*id))?;
    // ICONDIR + one ICONDIRENTRY (the group entry's first 8 bytes, then size
    // and offset) + the image.
    let mut ico = vec![0, 0, 1, 0, 1, 0];
    ico.extend_from_slice(entry);
    ico.extend_from_slice(&(image.len() as u32).to_le_bytes());
    ico.extend_from_slice(&22u32.to_le_bytes());
    ico.extend_from_slice(image);
    Some(ico)
}

/// The icon of an executable on disk.
pub fn icon_from_exe(path: &Path) -> Option<Vec<u8>> {
    if std::fs::metadata(path).ok()?.len() > MAX_EXE {
        return None;
    }
    icon_from_pe(&std::fs::read(path).ok()?)
}

/// Decode an `.ico` file to `([width, height], RGBA bytes)`.
pub fn decode_icon(ico: &[u8]) -> Option<([usize; 2], Vec<u8>)> {
    let image = image::load_from_memory_with_format(ico, image::ImageFormat::Ico)
        .ok()?
        .into_rgba8();
    Some((
        [image.width() as usize, image.height() as usize],
        image.into_raw(),
    ))
}
