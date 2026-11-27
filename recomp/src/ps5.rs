//! What a PS5 executable (a decrypted ELF) asks of the system: the modules and libraries it imports and the
//! symbols it takes from each, read from its SCE dynamic-linking data. Symbols are named by NID (a hash of the
//! function's name, written in base64), followed by `#library#module` ids. Nothing of a title is kept here: the
//! scout reads the user's own copy and prints counts.

use std::collections::BTreeMap;

const PT_DYNAMIC: u32 = 2;
const PT_SCE_DYNLIBDATA: u32 = 0x6100_0000;

const DT_NEEDED: u64 = 1;
const DT_STRTAB: u64 = 5;
const DT_SYMTAB: u64 = 6;
const DT_STRSZ: u64 = 10;
const DT_SCE_NEEDED_MODULE: u64 = 0x6100_000f;
const DT_SCE_IMPORT_LIB: u64 = 0x6100_0015;
const DT_SCE_STRTAB: u64 = 0x6100_0035;
const DT_SCE_STRSZ: u64 = 0x6100_0037;
const DT_SCE_SYMTAB: u64 = 0x6100_0039;
const DT_SCE_SYMTABSZ: u64 = 0x6100_003f;
// The PS5's tags for needed modules and imported libraries; its tables are virtual addresses in a loadable
// segment (it has no dynlib data segment).
const DT_SCE_NEEDED_MODULE_PS5: u64 = 0x6100_0045;
const DT_SCE_IMPORT_LIB_PS5: u64 = 0x6100_0049;
const DT_HASH: u64 = 4;
const DT_RELA: u64 = 7;
const DT_JMPREL: u64 = 0x17;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Imports {
    /// Modules by id (the low 16 bits of the tag's value), with their names.
    pub modules: BTreeMap<u64, String>,
    /// Libraries by id, with their names.
    pub libraries: BTreeMap<u64, String>,
    /// Imported symbols per library id (undefined symbols named `nid#lib#mod`).
    pub symbols: BTreeMap<u64, Vec<String>>,
    /// Plain ELF DT_NEEDED entries, when the image uses them.
    pub needed: Vec<String>,
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

fn cstr(d: &[u8], o: usize) -> Option<String> {
    let s = d.get(o..)?;
    let end = s.iter().position(|&b| b == 0)?;
    Some(String::from_utf8_lossy(&s[..end]).into_owned())
}

/// A library or module id in an SCE symbol suffix: base64-like digits (A-Z, a-z, 0-9, +, -).
fn id_of(s: &str) -> Option<u64> {
    let mut v = 0u64;
    for c in s.bytes() {
        let d = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'-' | b'/' => 63,
            _ => return None,
        };
        v = v.checked_mul(64)?.checked_add(u64::from(d))?;
    }
    Some(v)
}

pub fn imports(data: &[u8]) -> Result<Imports, String> {
    if !crate::elf::is_elf(data) {
        return Err(if crate::elf::is_self(data) {
            "a signed (encrypted) executable: it must be decrypted on its console first".into()
        } else {
            "not an ELF".into()
        });
    }
    let phoff = u64_at(data, 32).ok_or("short header")? as usize;
    let phentsize = u16_at(data, 54).ok_or("short header")? as usize;
    let phnum = u16_at(data, 56).ok_or("short header")? as usize;
    let (mut dynamic, mut dynlib) = (None, None);
    let mut loads: Vec<(u64, u64, u64)> = Vec::new();
    for k in 0..phnum {
        let p = phoff + k * phentsize;
        let kind = u32_at(data, p).ok_or("short program header")?;
        let offset = u64_at(data, p + 8).ok_or("short program header")? as usize;
        let filesz = u64_at(data, p + 32).ok_or("short program header")? as usize;
        let vaddr = u64_at(data, p + 16).ok_or("short program header")?;
        if kind == 1 {
            loads.push((vaddr, offset as u64, filesz as u64));
        }
        match kind {
            PT_DYNAMIC => dynamic = Some((offset, filesz)),
            PT_SCE_DYNLIBDATA => dynlib = Some(offset),
            _ => {}
        }
    }
    let (doff, dsize) = dynamic.ok_or("no dynamic segment (a static executable)")?;
    let mut tags = Vec::new();
    for k in 0..dsize / 16 {
        let tag = u64_at(data, doff + 16 * k).ok_or("short dynamic segment")?;
        let val = u64_at(data, doff + 16 * k + 8).ok_or("short dynamic segment")?;
        if tag == 0 {
            break;
        }
        tags.push((tag, val));
    }
    let find = |t: u64| tags.iter().find(|(tag, _)| *tag == t).map(|&(_, v)| v);
    // PS4: the string and symbol tables are offsets into the SCE dynlib data segment. PS5: virtual addresses in
    // a loadable segment.
    let at = |va: u64| -> Option<usize> {
        loads.iter().find(|&&(v, _, f)| va >= v && va < v + f).map(|&(v, o, _)| (o + va - v) as usize)
    };
    let table = |sce: u64, plain: u64| -> Option<usize> {
        match dynlib {
            Some(base) => Some(base + find(sce).or_else(|| find(plain))? as usize),
            None => at(find(plain)?),
        }
    };
    let strtab = table(DT_SCE_STRTAB, DT_STRTAB).ok_or("no string table")?;
    let strsz = find(DT_SCE_STRSZ).or_else(|| find(DT_STRSZ)).unwrap_or(0) as usize;
    let string = |o: u64| -> Option<String> {
        if strsz > 0 && o as usize >= strsz {
            return None;
        }
        cstr(data, strtab + o as usize)
    };
    let mut out = Imports::default();
    for &(tag, val) in &tags {
        match tag {
            // value: name offset in the low 32 bits, id in the top 16
            DT_SCE_NEEDED_MODULE | DT_SCE_NEEDED_MODULE_PS5 => {
                if let Some(n) = string(val & 0xffff_ffff) {
                    out.modules.insert(val >> 48, n);
                }
            }
            DT_SCE_IMPORT_LIB | DT_SCE_IMPORT_LIB_PS5 => {
                if let Some(n) = string(val & 0xffff_ffff) {
                    out.libraries.insert(val >> 48, n);
                }
            }
            DT_NEEDED => {
                if let Some(n) = string(val) {
                    out.needed.push(n);
                }
            }
            _ => {}
        }
    }
    let symtab = table(DT_SCE_SYMTAB, DT_SYMTAB);
    // PS5 gives no size: the symbols run to the next table that follows them.
    let symsz = match find(DT_SCE_SYMTABSZ) {
        Some(n) => n as usize,
        None => find(DT_SYMTAB)
            .and_then(|sym| {
                [DT_STRTAB, DT_RELA, DT_JMPREL, DT_HASH]
                    .iter()
                    .filter_map(|&t| find(t))
                    .filter(|&a| a > sym)
                    .min()
                    .map(|next| (next - sym) as usize)
            })
            .unwrap_or(0),
    };
    if let Some(st) = symtab {
        for k in 0..symsz / 24 {
            let e = st + 24 * k;
            let name = u32_at(data, e).ok_or("short symbol table")?;
            let shndx = u16_at(data, e + 6).ok_or("short symbol table")?;
            if shndx != 0 || name == 0 {
                continue; // defined here, or unnamed
            }
            let Some(n) = string(u64::from(name)) else { continue };
            let mut parts = n.split('#');
            let (Some(nid), Some(lib)) = (parts.next(), parts.next()) else { continue };
            if let Some(lib) = id_of(lib) {
                out.symbols.entry(lib).or_default().push(nid.to_string());
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal ELF with a dynamic segment and an SCE dynlib data segment: one needed module, one imported
    /// library, and three symbols: two imported from that library, one defined.
    fn sample() -> Vec<u8> {
        let mut d = vec![0u8; 0x800];
        d[..4].copy_from_slice(b"\x7fELF");
        d[4] = 2;
        d[5] = 1;
        d[18] = 0x3E;
        d[32..40].copy_from_slice(&64u64.to_le_bytes());
        d[54..56].copy_from_slice(&56u16.to_le_bytes());
        d[56..58].copy_from_slice(&2u16.to_le_bytes());
        let ph = |d: &mut Vec<u8>, at: usize, kind: u32, off: u64, size: u64| {
            d[at..at + 4].copy_from_slice(&kind.to_le_bytes());
            d[at + 8..at + 16].copy_from_slice(&off.to_le_bytes());
            d[at + 32..at + 40].copy_from_slice(&size.to_le_bytes());
        };
        ph(&mut d, 64, PT_DYNAMIC, 0x200, 0x80);
        ph(&mut d, 120, PT_SCE_DYNLIBDATA, 0x400, 0x400);
        // strings at dynlib + 0
        let strings = b"\0libKernel\0libkernel\0AbCdEfGhIjK#B#B\0ZzZzZzZzZzZ#B#B\0ownSymbol\0";
        d[0x400..0x400 + strings.len()].copy_from_slice(strings);
        let at = |s: &str| strings.windows(s.len()).position(|w| w == s.as_bytes()).unwrap() as u64;
        // symbols at dynlib + 0x100: two undefined, one defined
        let sym = |d: &mut Vec<u8>, k: usize, name: u64, shndx: u16| {
            let e = 0x500 + 24 * k;
            d[e..e + 4].copy_from_slice(&(name as u32).to_le_bytes());
            d[e + 6..e + 8].copy_from_slice(&shndx.to_le_bytes());
        };
        sym(&mut d, 0, at("AbCdEfGhIjK#B#B"), 0);
        sym(&mut d, 1, at("ZzZzZzZzZzZ#B#B"), 0);
        sym(&mut d, 2, at("ownSymbol"), 5);
        let tag = |d: &mut Vec<u8>, k: usize, t: u64, v: u64| {
            d[0x200 + 16 * k..0x208 + 16 * k].copy_from_slice(&t.to_le_bytes());
            d[0x208 + 16 * k..0x210 + 16 * k].copy_from_slice(&v.to_le_bytes());
        };
        tag(&mut d, 0, DT_SCE_STRTAB, 0);
        tag(&mut d, 1, DT_SCE_STRSZ, strings.len() as u64);
        tag(&mut d, 2, DT_SCE_SYMTAB, 0x100);
        tag(&mut d, 3, DT_SCE_SYMTABSZ, 72);
        tag(&mut d, 4, DT_SCE_NEEDED_MODULE, (1u64 << 48) | at("libKernel"));
        tag(&mut d, 5, DT_SCE_IMPORT_LIB, (1u64 << 48) | at("libkernel"));
        d
    }

    #[test]
    fn imports_come_by_library_with_their_nids() {
        let i = imports(&sample()).unwrap();
        assert_eq!(i.modules.get(&1).map(String::as_str), Some("libKernel"));
        assert_eq!(i.libraries.get(&1).map(String::as_str), Some("libkernel"));
        assert_eq!(i.symbols.get(&1), Some(&vec!["AbCdEfGhIjK".to_string(), "ZzZzZzZzZzZ".to_string()]));
        assert_eq!(i.symbols.len(), 1); // the defined symbol is not an import
    }

    #[test]
    fn an_encrypted_executable_is_refused_with_a_reason() {
        let e = imports(&[0x4F, 0x15, 0x3D, 0x1D, 0, 0, 0, 0]).unwrap_err();
        assert!(e.contains("decrypted"));
        assert_eq!(id_of("B"), Some(1));
        assert_eq!(id_of("BA"), Some(64));
    }
}
