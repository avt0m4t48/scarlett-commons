//! Synthetic test functions, assembled here and translated by commons-recomp-emit into $OUT_DIR/synthetic.rs, so
//! tests/emit_native.rs can run each one natively and translated on the same inputs. No title code is involved.

use iced_x86::code_asm::*;

const IP: u64 = 0x1000_0000;

fn assemble(build: impl FnOnce(&mut CodeAssembler) -> Result<(), IcedError>) -> Vec<u8> {
    let mut a = CodeAssembler::new(64).expect("assembler");
    build(&mut a).expect("assemble");
    a.assemble(IP).expect("encode")
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=emit/src/lib.rs");
    let cases: Vec<(&str, u64, Vec<u8>)> = vec![
        // (name, flags compared as an RFLAGS mask: CF 0x1, ZF 0x40, SF 0x80, OF 0x800, code)
        ("arith", 0x8c1, assemble(|a| {
            a.mov(rax, rcx)?; a.add(rax, rdx)?; a.sub(rax, r8)?; a.xor(r9, rax)?; a.and(r9d, 0xff00ff)?;
            a.or(rax, r9)?; a.add(r10d, r11d)?; a.ret()
        })),
        ("flags", 0x8c1, assemble(|a| {
            a.cmp(rcx, rdx)?; a.setb(al)?; a.setl(r8b)?; a.setg(r9b)?; a.sete(r10b)?; a.seta(r11b)?;
            a.test(rcx, rcx)?; a.sets(bl)?; a.cmp(ecx, 0x7fff)?; a.ret()
        })),
        ("loop", 0x8c1, assemble(|a| {
            let mut top = a.create_label();
            let mut done = a.create_label();
            a.xor(eax, eax)?; a.test(rcx, rcx)?; a.jz(done)?;
            a.set_label(&mut top)?;
            a.add(rax, qword_ptr(rdx))?; a.add(rdx, 8)?; a.dec(rcx)?; a.jnz(top)?;
            a.set_label(&mut done)?;
            a.ret()
        })),
        ("shift", 0xc1, assemble(|a| {
            a.mov(rax, rcx)?; a.shl(rax, 1)?; a.shr(rdx, 1)?; a.sar(r8, 1)?; a.mov(r9, rdx)?; a.shl(r9, cl)?;
            a.sar(r10d, cl)?; a.shr(r11d, 7)?; a.ret()
        })),
        ("memory", 0x8c1, assemble(|a| {
            a.mov(qword_ptr(rdx), rcx)?; a.mov(eax, dword_ptr(rdx))?; a.movzx(r8, byte_ptr(rdx + 1))?;
            a.movsx(r9, word_ptr(rdx + 2))?; a.lea(r10, qword_ptr(rdx + rcx * 4 + 0x10))?; a.push(rcx)?;
            a.pop(r11)?; a.mov(byte_ptr(rdx + 9), r8b)?; a.movsxd(rbx, dword_ptr(rdx + 4))?; a.ret()
        })),
        ("select", 0x8c1, assemble(|a| {
            a.cmp(rcx, rdx)?; a.cmovg(rax, rcx)?; a.cmovle(rax, rdx)?; a.cmovb(esi, edi)?; a.neg(r8)?;
            a.not(r9)?; a.inc(r10)?; a.dec(r11d)?; a.xchg(r12, r13)?; a.ret()
        })),
    ];
    let mut out = String::from(commons_recomp_emit::PRELUDE);
    out += "pub struct Case { pub name: &'static str, pub flags: u64, pub code: &'static [u8], pub translated: unsafe fn(&mut Ctx) }\n";
    let mut table = String::from("pub const CASES: &[Case] = &[\n");
    for (name, flags, code) in &cases {
        let f = format!("t_{name}");
        out += &commons_recomp_emit::emit_function(&f, code, IP)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        out += &format!("pub const C_{}: &[u8] = &{code:?};\n", name.to_uppercase());
        table += &format!(
            "    Case {{ name: \"{name}\", flags: {flags:#x}, code: C_{}, translated: {f} }},\n",
            name.to_uppercase()
        );
    }
    table += "];\n";
    out += &table;
    let dir = std::env::var("OUT_DIR").expect("OUT_DIR");
    std::fs::write(std::path::Path::new(&dir).join("synthetic.rs"), out).expect("write");
}
