//! The census of a build's functions for track B: every function the executable's `.pdata` lists, decoded with
//! iced-x86 (MIT), counted against what the track B emitter covers. It reads the file and never runs it; the report
//! holds counts only, no code.
//!
//! "Covered" is what the track B emitter translates (commons-recomp-emit, the one definition): today its v1 integer
//! subset, without calls. A function is fully covered when every instruction is; a leaf is one without calls.

use std::collections::BTreeMap;

use iced_x86::{Decoder, DecoderOptions, FlowControl, Instruction};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeFunction {
    pub begin: u32,
    pub end: u32,
}

/// A PE32+ image as a file on disk: sections to map RVAs, the image base and the exception directory.
pub struct Image<'a> {
    data: &'a [u8],
    pub image_base: u64,
    pub size_of_image: u32,
    pub size_of_headers: u32,
    sections: Vec<(u32, u32, u32, u32)>, // virtual address, virtual size, raw offset, raw size
    exception: (u32, u32),
    /// The import address table (RVA, size): what the image's calls to other modules go through.
    pub iat: (u32, u32),
}

fn u16_at(d: &[u8], o: usize) -> Option<u16> {
    Some(u16::from_le_bytes(d.get(o..o + 2)?.try_into().ok()?))
}
fn u32_at(d: &[u8], o: usize) -> Option<u32> {
    Some(u32::from_le_bytes(d.get(o..o + 4)?.try_into().ok()?))
}
fn u64_at(d: &[u8], o: usize) -> Option<u64> {
    Some(u64::from_le_bytes(d.get(o..o + 8)?.try_into().ok()?))
}

impl<'a> Image<'a> {
    pub fn parse(data: &'a [u8]) -> Result<Image<'a>, String> {
        let pe = u32_at(data, 0x3C).ok_or("no DOS header")? as usize;
        if data.get(pe..pe + 4) != Some(b"PE\0\0") {
            return Err("not a PE image".into());
        }
        let coff = pe + 4;
        let sections_n = u16_at(data, coff + 2).ok_or("short COFF header")? as usize;
        let opt_size = u16_at(data, coff + 16).ok_or("short COFF header")? as usize;
        let opt = coff + 20;
        if u16_at(data, opt) != Some(0x20B) {
            return Err("not PE32+ (64-bit)".into());
        }
        let image_base = u64_at(data, opt + 24).ok_or("short optional header")?;
        let size_of_image = u32_at(data, opt + 56).ok_or("short optional header")?;
        let size_of_headers = u32_at(data, opt + 60).ok_or("short optional header")?;
        let dirs = u32_at(data, opt + 108).ok_or("short optional header")?;
        let exception = if dirs > 3 {
            (u32_at(data, opt + 112 + 24).unwrap_or(0), u32_at(data, opt + 112 + 28).unwrap_or(0))
        } else {
            (0, 0)
        };
        let iat = if dirs > 12 {
            (u32_at(data, opt + 112 + 96).unwrap_or(0), u32_at(data, opt + 112 + 100).unwrap_or(0))
        } else {
            (0, 0)
        };
        let table = opt + opt_size;
        let mut sections = Vec::with_capacity(sections_n);
        for i in 0..sections_n {
            let s = table + 40 * i;
            sections.push((
                u32_at(data, s + 12).ok_or("short section table")?,
                u32_at(data, s + 8).ok_or("short section table")?,
                u32_at(data, s + 20).ok_or("short section table")?,
                u32_at(data, s + 16).ok_or("short section table")?,
            ));
        }
        Ok(Image { data, image_base, size_of_image, size_of_headers, sections, exception, iat })
    }

    /// The file bytes of [rva, rva + len), when they lie inside one section's raw data.
    pub fn bytes(&self, rva: u32, len: u32) -> Option<&'a [u8]> {
        for &(va, vsize, raw, rsize) in &self.sections {
            let span = vsize.max(rsize);
            if rva >= va && rva.checked_add(len)? <= va.checked_add(span)? {
                let off = (raw as usize).checked_add((rva - va) as usize)?;
                let end = off.checked_add(len as usize)?;
                if (rva - va).checked_add(len)? > rsize {
                    return None; // past the raw data (zero-filled in memory, not in the file)
                }
                return self.data.get(off..end);
            }
        }
        None
    }

    /// Copies the headers and every section's raw data to `dest` at their RVAs, as a loader maps the image (no
    /// relocations, no imports). `dest` must hold `size_of_image` bytes.
    pub fn map_into(&self, dest: &mut [u8]) {
        let h = (self.size_of_headers as usize).min(self.data.len()).min(dest.len());
        dest[..h].copy_from_slice(&self.data[..h]);
        for &(va, _vsize, raw, rsize) in &self.sections {
            let (va, raw, rsize) = (va as usize, raw as usize, rsize as usize);
            let n = rsize.min(self.data.len().saturating_sub(raw)).min(dest.len().saturating_sub(va));
            if n > 0 {
                dest[va..va + n].copy_from_slice(&self.data[raw..raw + n]);
            }
        }
    }

    /// Every RUNTIME_FUNCTION in `.pdata`, sorted, without empty or reversed entries.
    pub fn functions(&self) -> Vec<RuntimeFunction> {
        let (rva, size) = self.exception;
        let Some(table) = self.bytes(rva, size) else { return Vec::new() };
        let mut out: Vec<RuntimeFunction> = table
            .chunks_exact(12)
            .filter_map(|e| {
                let begin = u32::from_le_bytes(e[0..4].try_into().ok()?);
                let end = u32::from_le_bytes(e[4..8].try_into().ok()?);
                (begin != 0 && end > begin).then_some(RuntimeFunction { begin, end })
            })
            .collect();
        out.sort_by_key(|f| f.begin);
        out
    }
}

/// Whether the track B emitter covers this instruction: the emitter's own definition (commons-recomp-emit).
pub fn covered(i: &Instruction) -> bool {
    commons_recomp_emit::supported(i)
}

#[derive(Debug, Default, Clone, Serialize, PartialEq, Eq)]
pub struct Census {
    pub functions: u64,
    pub code_bytes: u64,
    pub instructions: u64,
    pub covered_instructions: u64,
    pub fully_covered_functions: u64,
    /// Fully covered and without calls: the first track B can take with generated inputs alone.
    pub fully_covered_leaves: u64,
    pub decode_errors: u64,
    /// Functions whose bytes are not in the file (outside every section's raw data).
    pub unreadable_functions: u64,
    /// The mnemonics most often left uncovered, with their counts (the work list for the emitter).
    pub top_uncovered: Vec<(String, u64)>,
}

impl Census {
    pub fn covered_percent(&self) -> f64 {
        if self.instructions == 0 { 0.0 } else { 100.0 * self.covered_instructions as f64 / self.instructions as f64 }
    }
}

/// The census of one function's bytes at `ip`: (instructions, covered, decode errors, fully covered, has calls).
pub fn census_code(code: &[u8], ip: u64, uncovered: &mut BTreeMap<String, u64>) -> (u64, u64, u64, bool, bool) {
    let mut decoder = Decoder::with_ip(64, code, ip, DecoderOptions::NONE);
    let (mut n, mut ok, mut errors, mut calls) = (0u64, 0u64, 0u64, false);
    let mut i = Instruction::default();
    while decoder.can_decode() {
        decoder.decode_out(&mut i);
        n += 1;
        if i.is_invalid() {
            errors += 1;
            *uncovered.entry("(undecodable)".into()).or_default() += 1;
            continue;
        }
        if i.flow_control() == FlowControl::Call || i.flow_control() == FlowControl::IndirectCall {
            calls = true;
        }
        if covered(&i) {
            ok += 1;
        } else {
            *uncovered.entry(format!("{:?}", i.mnemonic()).to_lowercase()).or_default() += 1;
        }
    }
    (n, ok, errors, n > 0 && ok == n, calls)
}

pub fn census(image: &Image<'_>, top: usize) -> Census {
    let mut c = Census::default();
    let mut uncovered = BTreeMap::new();
    for f in image.functions() {
        c.functions += 1;
        let len = f.end - f.begin;
        c.code_bytes += u64::from(len);
        let Some(code) = image.bytes(f.begin, len) else {
            c.unreadable_functions += 1;
            continue;
        };
        let (n, ok, errors, full, calls) = census_code(code, image.image_base + u64::from(f.begin), &mut uncovered);
        c.instructions += n;
        c.covered_instructions += ok;
        c.decode_errors += errors;
        if full {
            c.fully_covered_functions += 1;
            if !calls {
                c.fully_covered_leaves += 1;
            }
        }
    }
    let mut top_list: Vec<(String, u64)> = uncovered.into_iter().collect();
    top_list.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    top_list.truncate(top);
    c.top_uncovered = top_list;
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    fn code(bytes: &[u8]) -> (u64, u64, u64, bool, bool) {
        census_code(bytes, 0x1_4000_1000, &mut BTreeMap::new())
    }

    #[test]
    fn integer_leaf_code_is_covered() {
        // mov rax, rcx; add rax, rdx; ret
        assert_eq!(code(&[0x48, 0x89, 0xC8, 0x48, 0x01, 0xD0, 0xC3]), (3, 3, 0, true, false));
    }

    #[test]
    fn a_call_is_covered_but_not_a_leaf() {
        // call rel32; ret: calls go through call_native, so the function is covered, and it is no leaf
        let (n, ok, _, full, calls) = code(&[0xE8, 0, 0, 0, 0, 0xC3]);
        assert_eq!((n, ok, full, calls), (2, 2, true, true));
    }

    #[test]
    fn simd_and_indirect_flow_are_not_covered() {
        // movaps xmm0, xmm1; jmp rax; ret
        let mut u = BTreeMap::new();
        let (n, ok, _, full, _) = census_code(&[0x0F, 0x28, 0xC1, 0xFF, 0xE0, 0xC3], 0x1000, &mut u);
        assert_eq!((n, ok, full), (3, 1, false));
        assert!(u.contains_key("movaps") && u.contains_key("jmp"));
    }

    #[test]
    fn gs_reads_of_the_thread_block_are_covered_and_other_gs_access_is_not() {
        // mov rax, gs:[0x58] (the thread's TLS array)
        let (_, ok, _, full, _) = code(&[0x65, 0x48, 0x8B, 0x04, 0x25, 0x58, 0, 0, 0]);
        assert_eq!((ok, full), (1, true));
        // mov gs:[0x58], rax; mov rax, gs:[rcx]
        let (_, ok, _, full, _) = code(&[0x65, 0x48, 0x89, 0x04, 0x25, 0x58, 0, 0, 0]);
        assert_eq!((ok, full), (0, false));
        let (_, ok, _, full, _) = code(&[0x65, 0x48, 0x8B, 0x01]);
        assert_eq!((ok, full), (0, false));
    }

    /// A minimal PE32+ with one section holding two functions and a .pdata that lists them.
    fn synthetic_pe() -> Vec<u8> {
        let mut d = vec![0u8; 0x600];
        d[0x3C..0x40].copy_from_slice(&0x80u32.to_le_bytes());
        d[0x80..0x84].copy_from_slice(b"PE\0\0");
        let coff = 0x84;
        d[coff + 2..coff + 4].copy_from_slice(&1u16.to_le_bytes()); // one section
        d[coff + 16..coff + 18].copy_from_slice(&240u16.to_le_bytes()); // optional header size
        let opt = coff + 20;
        d[opt..opt + 2].copy_from_slice(&0x20Bu16.to_le_bytes());
        d[opt + 24..opt + 32].copy_from_slice(&0x1_4000_0000u64.to_le_bytes());
        d[opt + 108..opt + 112].copy_from_slice(&16u32.to_le_bytes());
        d[opt + 136..opt + 140].copy_from_slice(&0x1100u32.to_le_bytes()); // exception directory rva
        d[opt + 140..opt + 144].copy_from_slice(&24u32.to_le_bytes()); // two entries
        let s = opt + 240;
        d[s + 8..s + 12].copy_from_slice(&0x400u32.to_le_bytes()); // virtual size
        d[s + 12..s + 16].copy_from_slice(&0x1000u32.to_le_bytes()); // virtual address
        d[s + 16..s + 20].copy_from_slice(&0x400u32.to_le_bytes()); // raw size
        d[s + 20..s + 24].copy_from_slice(&0x200u32.to_le_bytes()); // raw offset
        // 0x1000: mov rax, rcx; ret (covered leaf). 0x1010: movaps xmm0, xmm1; ret (not covered).
        d[0x200..0x204].copy_from_slice(&[0x48, 0x89, 0xC8, 0xC3]);
        d[0x210..0x214].copy_from_slice(&[0x0F, 0x28, 0xC1, 0xC3]);
        for (i, (b, e)) in [(0x1000u32, 0x1004u32), (0x1010, 0x1014)].iter().enumerate() {
            let at = 0x300 + 12 * i;
            d[at..at + 4].copy_from_slice(&b.to_le_bytes());
            d[at + 4..at + 8].copy_from_slice(&e.to_le_bytes());
        }
        d
    }

    #[test]
    fn a_synthetic_image_is_counted() {
        let pe = synthetic_pe();
        let img = Image::parse(&pe).unwrap();
        assert_eq!(img.functions().len(), 2);
        let c = census(&img, 10);
        assert_eq!((c.functions, c.instructions, c.covered_instructions), (2, 4, 3));
        assert_eq!((c.fully_covered_functions, c.fully_covered_leaves), (1, 1));
        assert_eq!(c.top_uncovered, vec![("movaps".to_string(), 1)]);
    }

    #[test]
    fn not_a_pe_is_refused() {
        assert!(Image::parse(&[0u8; 64]).is_err());
    }
}
