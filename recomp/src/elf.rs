//! ELF64 executables (the PS5's format, once decrypted): the loadable segments, and the functions from the unwind
//! data (`.eh_frame` through `.eh_frame_hdr`), the counterpart of a PE's `.pdata`. Nothing of a title is kept here.

use crate::census::RuntimeFunction;

/// What the image needs of an ELF: the base, the size, the segments as (virtual address, memory size, file offset,
/// file size) relative to the base, and the functions.
pub struct Elf {
    pub base: u64,
    pub size: u32,
    pub segments: Vec<(u32, u32, u32, u32)>,
    pub functions: Vec<RuntimeFunction>,
}

const PT_LOAD: u32 = 1;
const PT_GNU_EH_FRAME: u32 = 0x6474_e550;

fn u16_at(d: &[u8], o: usize) -> Option<u16> {
    Some(u16::from_le_bytes(d.get(o..o + 2)?.try_into().ok()?))
}
fn u32_at(d: &[u8], o: usize) -> Option<u32> {
    Some(u32::from_le_bytes(d.get(o..o + 4)?.try_into().ok()?))
}
fn u64_at(d: &[u8], o: usize) -> Option<u64> {
    Some(u64::from_le_bytes(d.get(o..o + 8)?.try_into().ok()?))
}

pub fn is_elf(data: &[u8]) -> bool {
    data.get(..4) == Some(b"\x7fELF")
}

/// A signed (still encrypted) PS5 executable: it must be decrypted on the console it was dumped from first.
pub fn is_self(data: &[u8]) -> bool {
    data.get(..4) == Some(&[0x4F, 0x15, 0x3D, 0x1D])
}

pub fn parse(data: &[u8]) -> Result<Elf, String> {
    if !is_elf(data) {
        return Err("not an ELF".into());
    }
    if data.get(4) != Some(&2) || data.get(5) != Some(&1) {
        return Err("not a 64-bit little-endian ELF".into());
    }
    if u16_at(data, 18) != Some(0x3E) {
        return Err("not x86-64".into());
    }
    let phoff = u64_at(data, 32).ok_or("short header")? as usize;
    let phentsize = u16_at(data, 54).ok_or("short header")? as usize;
    let phnum = u16_at(data, 56).ok_or("short header")? as usize;
    let mut loads = Vec::new();
    let mut eh = None;
    for k in 0..phnum {
        let p = phoff + k * phentsize;
        let kind = u32_at(data, p).ok_or("short program header")?;
        let offset = u64_at(data, p + 8).ok_or("short program header")?;
        let vaddr = u64_at(data, p + 16).ok_or("short program header")?;
        let filesz = u64_at(data, p + 32).ok_or("short program header")?;
        let memsz = u64_at(data, p + 40).ok_or("short program header")?;
        match kind {
            PT_LOAD => loads.push((vaddr, memsz, offset, filesz)),
            PT_GNU_EH_FRAME => eh = Some(vaddr),
            _ => {}
        }
    }
    if loads.is_empty() {
        return Err("no loadable segment".into());
    }
    let base = loads.iter().map(|l| l.0).min().unwrap_or(0) & !0xfff;
    let end = loads.iter().map(|l| l.0 + l.1).max().unwrap_or(0);
    let size = u32::try_from((end - base + 0xfff) & !0xfff).map_err(|_| "image larger than 4 GiB")?;
    let segments: Vec<(u32, u32, u32, u32)> = loads
        .iter()
        .map(|&(v, m, o, f)| ((v - base) as u32, m as u32, o as u32, f as u32))
        .collect();
    let read = |va: u64, len: usize| -> Option<&[u8]> {
        let rva = va.checked_sub(base)?;
        segments.iter().find_map(|&(v, _m, o, f)| {
            let (v, o, f) = (u64::from(v), u64::from(o), u64::from(f));
            (rva >= v && rva + len as u64 <= v + f).then(|| data.get((o + rva - v) as usize..(o + rva - v) as usize + len))?
        })
    };
    let functions = match eh {
        Some(hdr) => functions(hdr, base, &read).unwrap_or_default(),
        None => Vec::new(),
    };
    Ok(Elf { base, size, segments, functions })
}

/// A pointer in a DWARF exception-header encoding, read at virtual address `at` (for pc-relative) with `data` the
/// table's base (for data-relative); the value and its size.
fn pointer<'d>(enc: u8, at: u64, data_base: u64, read: &dyn Fn(u64, usize) -> Option<&'d [u8]>) -> Option<(u64, usize)> {
    let (raw, size): (i64, usize) = match enc & 0x0f {
        0x00 => (u64::from_le_bytes(read(at, 8)?.try_into().ok()?) as i64, 8),
        0x03 => (i64::from(u32::from_le_bytes(read(at, 4)?.try_into().ok()?)), 4),
        0x0b => (i64::from(i32::from_le_bytes(read(at, 4)?.try_into().ok()?)), 4),
        0x04 | 0x0c => (i64::from_le_bytes(read(at, 8)?.try_into().ok()?), 8),
        0x02 => (i64::from(u16::from_le_bytes(read(at, 2)?.try_into().ok()?)), 2),
        0x0a => (i64::from(i16::from_le_bytes(read(at, 2)?.try_into().ok()?)), 2),
        _ => return None,
    };
    let value = match enc & 0x70 {
        0x00 => raw as u64,
        0x10 => at.wrapping_add(raw as u64),
        0x30 => data_base.wrapping_add(raw as u64),
        _ => return None,
    };
    Some((value, size))
}

fn uleb<'d>(read: &dyn Fn(u64, usize) -> Option<&'d [u8]>, at: &mut u64) -> Option<u64> {
    let mut v = 0u64;
    for shift in (0..64).step_by(7) {
        let b = read(*at, 1)?[0];
        *at += 1;
        v |= u64::from(b & 0x7f) << shift;
        if b & 0x80 == 0 {
            return Some(v);
        }
    }
    None
}

/// The FDE pointer encoding a CIE's augmentation names ('R'), absolute pointers when it names none.
fn cie_encoding<'d>(cie: u64, read: &dyn Fn(u64, usize) -> Option<&'d [u8]>) -> Option<u8> {
    let mut at = cie + 8; // length, CIE id
    let version = read(at, 1)?[0];
    at += 1;
    let mut aug = Vec::new();
    loop {
        let c = read(at, 1)?[0];
        at += 1;
        if c == 0 {
            break;
        }
        aug.push(c);
    }
    if aug.first() != Some(&b'z') {
        return Some(0);
    }
    uleb(read, &mut at)?; // code alignment
    uleb(read, &mut at)?; // data alignment (the sign does not matter to skip it)
    if version == 1 {
        at += 1;
    } else {
        uleb(read, &mut at)?;
    }
    uleb(read, &mut at)?; // augmentation data length
    for &c in &aug[1..] {
        match c {
            b'R' => return Some(read(at, 1)?[0]),
            b'L' => at += 1,
            b'P' => {
                let enc = read(at, 1)?[0];
                at += 1;
                let (_, n) = pointer(enc, at, 0, read)?;
                at += n as u64;
            }
            _ => {}
        }
    }
    Some(0)
}

fn functions<'d>(hdr: u64, base: u64, read: &dyn Fn(u64, usize) -> Option<&'d [u8]>) -> Option<Vec<RuntimeFunction>> {
    let h = read(hdr, 4)?;
    let (version, ptr_enc, count_enc, table_enc) = (h[0], h[1], h[2], h[3]);
    if version != 1 {
        return None;
    }
    let mut at = hdr + 4;
    let (_, n) = pointer(ptr_enc, at, hdr, read)?;
    at += n as u64;
    let (count, n) = pointer(count_enc, at, hdr, read)?;
    at += n as u64;
    let mut out = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let (start, n) = pointer(table_enc, at, hdr, read)?;
        at += n as u64;
        let (fde, n) = pointer(table_enc, at, hdr, read)?;
        at += n as u64;
        // FDE: length, CIE pointer (relative to its own field), pc_begin, pc_range
        let cie_rel = u64::from(u32::from_le_bytes(read(fde + 4, 4)?.try_into().ok()?));
        let enc = cie_encoding(fde + 4 - cie_rel, read)?;
        let (_, n) = pointer(enc, fde + 8, 0, read)?;
        let (range, _) = pointer(enc & 0x0f, fde + 8 + n as u64, 0, read)?;
        let (begin, end) = (start.checked_sub(base)?, start.checked_sub(base)? + range);
        if range > 0 {
            out.push(RuntimeFunction { begin: u32::try_from(begin).ok()?, end: u32::try_from(end).ok()? });
        }
    }
    out.sort_by_key(|f| f.begin);
    out.dedup_by_key(|f| f.begin);
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal x86-64 ELF: one loadable segment at 0x400000 holding two functions, a CIE with FDE encoding
    /// pcrel|sdata4, two FDEs and an .eh_frame_hdr whose table is datarel|sdata4.
    fn sample() -> Vec<u8> {
        let mut d = vec![0u8; 0x1000];
        d[..4].copy_from_slice(b"\x7fELF");
        d[4] = 2;
        d[5] = 1;
        d[18] = 0x3E;
        d[32..40].copy_from_slice(&64u64.to_le_bytes()); // phoff
        d[54..56].copy_from_slice(&56u16.to_le_bytes());
        d[56..58].copy_from_slice(&2u16.to_le_bytes());
        let ph = |d: &mut Vec<u8>, at: usize, kind: u32, off: u64, va: u64, size: u64| {
            d[at..at + 4].copy_from_slice(&kind.to_le_bytes());
            d[at + 8..at + 16].copy_from_slice(&off.to_le_bytes());
            d[at + 16..at + 24].copy_from_slice(&va.to_le_bytes());
            d[at + 32..at + 40].copy_from_slice(&size.to_le_bytes());
            d[at + 40..at + 48].copy_from_slice(&size.to_le_bytes());
        };
        ph(&mut d, 64, PT_LOAD, 0, 0x400000, 0x1000);
        ph(&mut d, 120, PT_GNU_EH_FRAME, 0x800, 0x400800, 0x40);
        // .eh_frame at 0x900: CIE "zR" with R = 0x1b, then two FDEs
        let cie = [
            20u32.to_le_bytes().to_vec(), 0u32.to_le_bytes().to_vec(),
            vec![1, b'z', b'R', 0, 1, 0x78, 16, 1, 0x1b], vec![0; 7],
        ]
        .concat();
        d[0x900..0x900 + cie.len()].copy_from_slice(&cie);
        let fde = |d: &mut Vec<u8>, at: usize, start: u64, range: u32| {
            d[at..at + 4].copy_from_slice(&20u32.to_le_bytes());
            d[at + 4..at + 8].copy_from_slice(&((at + 4 - 0x900) as u32).to_le_bytes());
            let rel = (start as i64 - (0x400000 + at as i64 + 8)) as i32;
            d[at + 8..at + 12].copy_from_slice(&rel.to_le_bytes());
            d[at + 12..at + 16].copy_from_slice(&range.to_le_bytes());
        };
        fde(&mut d, 0x918, 0x400100, 0x40);
        fde(&mut d, 0x930, 0x400200, 0x80);
        // .eh_frame_hdr at 0x800: version 1, eh_frame_ptr pcrel|sdata4, count udata4, table datarel|sdata4
        d[0x800..0x804].copy_from_slice(&[1, 0x1b, 0x03, 0x3b]);
        d[0x804..0x808].copy_from_slice(&(0x100i32 - 4).to_le_bytes());
        d[0x808..0x80c].copy_from_slice(&2u32.to_le_bytes());
        let row = |d: &mut Vec<u8>, at: usize, start: i32, fde: i32| {
            d[at..at + 4].copy_from_slice(&start.to_le_bytes());
            d[at + 4..at + 8].copy_from_slice(&fde.to_le_bytes());
        };
        row(&mut d, 0x80c, 0x100 - 0x800, 0x918 - 0x800);
        row(&mut d, 0x814, 0x200 - 0x800, 0x930 - 0x800);
        d
    }

    #[test]
    fn functions_come_from_the_unwind_data() {
        let e = parse(&sample()).unwrap();
        assert_eq!(e.base, 0x400000);
        assert_eq!(e.size, 0x1000);
        assert_eq!(
            e.functions,
            vec![RuntimeFunction { begin: 0x100, end: 0x140 }, RuntimeFunction { begin: 0x200, end: 0x280 }]
        );
    }

    #[test]
    fn a_signed_executable_is_told_apart() {
        assert!(is_self(&[0x4F, 0x15, 0x3D, 0x1D, 0]));
        assert!(!is_elf(&[0x4F, 0x15, 0x3D, 0x1D]));
        assert!(parse(b"MZ\0\0").is_err());
    }
}
