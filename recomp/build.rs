//! Synthetic test functions, assembled here and translated by commons-recomp-emit into $OUT_DIR/synthetic.rs, so
//! tests/emit_native.rs can run each one natively and translated on the same inputs. No title code is involved.

use iced_x86::code_asm::*;

/// RVAs: the cases at 0x1000, the native callee the call cases call at 0x2000; the test maps them at a base.
const IP: u64 = 0x1000;
const CALLEE: u64 = 0x2000;

fn assemble_at(ip: u64, build: impl FnOnce(&mut CodeAssembler) -> Result<(), IcedError>) -> Vec<u8> {
    let mut a = CodeAssembler::new(64).expect("assembler");
    build(&mut a).expect("assemble");
    a.assemble(ip).expect("encode")
}

fn assemble(build: impl FnOnce(&mut CodeAssembler) -> Result<(), IcedError>) -> Vec<u8> {
    assemble_at(IP, build)
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=emit/src/lib.rs");
    let cases: Vec<(&str, u64, Vec<u8>)> = vec![
        // (name, flags compared as an RFLAGS mask: CF 0x1, PF 0x4, ZF 0x40, SF 0x80, OF 0x800, code)
        ("vucomiss", 0x8c5, assemble(|a| {
            let mut unordered = a.create_label();
            let mut after_register = a.create_label();
            let mut ordered = a.create_label();
            let mut finish = a.create_label();
            a.mov(eax, i32::MAX as u32)?; a.cmp(eax, -1)?; // start with SF and OF both set
            a.vucomiss(xmm0, xmm1)?; a.jp(unordered)?;
            a.mov(eax, 0)?; a.jmp(after_register)?;
            a.set_label(&mut unordered)?;
            a.mov(eax, 1)?;
            a.set_label(&mut after_register)?;
            a.setp(bl)?; a.setnp(r8b)?; a.cmovp(r9, r10)?; a.cmovnp(r11, r12)?; a.setb(cl)?; a.sete(r13b)?;
            a.vucomiss(xmm2, dword_ptr(rdx + 1))?; a.jnp(ordered)?;
            a.mov(esi, 1)?; a.jmp(finish)?;
            a.set_label(&mut ordered)?;
            a.mov(esi, 0)?;
            a.set_label(&mut finish)?;
            a.ret()
        })),
        ("arith", 0x8c5, assemble(|a| {
            a.mov(rax, rcx)?; a.add(rax, rdx)?; a.sub(rax, r8)?; a.xor(r9, rax)?; a.and(r9d, 0xff00ff)?;
            a.or(rax, r9)?; a.add(r10d, r11d)?; a.ret()
        })),
        ("flags", 0x8c5, assemble(|a| {
            a.cmp(rcx, rdx)?; a.setb(al)?; a.setl(r8b)?; a.setg(r9b)?; a.sete(r10b)?; a.seta(r11b)?;
            a.setp(r12b)?; a.test(rcx, rcx)?; a.sets(bl)?; a.setnp(r13b)?; a.cmp(ecx, 0x7fff)?; a.ret()
        })),
        ("loop", 0x8c5, assemble(|a| {
            let mut top = a.create_label();
            let mut done = a.create_label();
            a.xor(eax, eax)?; a.test(rcx, rcx)?; a.jz(done)?;
            a.set_label(&mut top)?;
            a.add(rax, qword_ptr(rdx))?; a.add(rdx, 8)?; a.dec(rcx)?; a.jnz(top)?;
            a.set_label(&mut done)?;
            a.ret()
        })),
        ("shift", 0xc5, assemble(|a| {
            a.mov(rax, rcx)?; a.shl(rax, 1)?; a.shr(rdx, 1)?; a.sar(r8, 1)?; a.mov(r9, rdx)?; a.shl(r9, cl)?;
            a.sar(r10d, cl)?; a.shr(r11d, 7)?; a.ret()
        })),
        ("memory", 0x8c1, assemble(|a| {
            a.mov(qword_ptr(rdx), rcx)?; a.mov(eax, dword_ptr(rdx))?; a.movzx(r8, byte_ptr(rdx + 1))?;
            a.movsx(r9, word_ptr(rdx + 2))?; a.lea(r10, qword_ptr(rdx + rcx * 4 + 0x10))?; a.push(rcx)?;
            a.pop(r11)?; a.mov(byte_ptr(rdx + 9), r8b)?; a.movsxd(rbx, dword_ptr(rdx + 4))?; a.ret()
        })),
        ("select", 0x8c5, assemble(|a| {
            a.cmp(rcx, rdx)?; a.cmovg(rax, rcx)?; a.cmovle(rax, rdx)?; a.cmovb(esi, edi)?; a.neg(r8)?;
            a.not(r9)?; a.inc(r10)?; a.dec(r11d)?; a.xchg(r12, r13)?; a.ret()
        })),
        ("imul", 0x801, assemble(|a| {
            a.imul_2(r11, qword_ptr(rdx))?; a.imul_2(rax, rcx)?; a.imul_2(edx, r8d)?; a.imul_3(r9, r10, 0x1234)?;
            a.imul_3(r12d, r13d, -7)?; a.imul_2(rsi, rdi)?; a.ret()
        })),
        ("vector", 0x8c1, assemble(|a| {
            a.vmovups(ymm0, ymmword_ptr(rdx))?; a.vmovups(xmm1, xmmword_ptr(rdx + 32))?; a.vaddps(ymm2, ymm0, ymm3)?;
            a.vmulps(xmm4, xmm1, xmm5)?; a.vxorps(ymm6, ymm6, ymm7)?; a.vsubss(xmm8, xmm9, dword_ptr(rdx + 4))?;
            a.vdivsd(xmm10, xmm11, xmm12)?; a.vbroadcastss(ymm13, dword_ptr(rdx + 8))?;
            a.vmovss(xmm14, dword_ptr(rdx + 12))?; a.vmovups(ymmword_ptr(rdx + 64), ymm2)?;
            a.vandps(xmm15, xmm0, xmm1)?; a.vmulpd(ymm3, ymm4, ymm5)?; a.vaddss(xmm5, xmm6, xmm7)?;
            a.vmovss_3(xmm9, xmm10, xmm11)?; a.vmovsd(qword_ptr(rdx + 128), xmm3)?; a.vpxor(xmm12, xmm12, xmm13)?;
            a.vmovaps(ymm11, ymm2)?; a.ret()
        })),
        ("zeroupper", 0x8c1, assemble(|a| {
            a.vmovups(ymm0, ymmword_ptr(rdx))?; a.vaddps(ymm1, ymm0, ymm0)?; a.vzeroupper()?; a.ret()
        })),
        ("call", 0x8c5, assemble(|a| {
            a.push(rbx)?; a.mov(rbx, rcx)?; a.mov(rcx, rdx)?; a.mov(rdx, rbx)?; a.test(r8, r8)?; a.call(CALLEE)?;
            a.setp(r12b)?; a.add(rax, rbx)?; a.pop(rbx)?; a.ret()
        })),
        ("call_indirect", 0x8c5, assemble(|a| {
            a.push(rbx)?; a.mov(rbx, rcx)?; a.test(r8, r8)?; a.call(r15)?;
            a.setp(r12b)?; a.sub(rax, rbx)?; a.mov(rcx, r9)?; a.pop(rbx)?; a.ret()
        })),
    ];
    // The native callee of the call cases: it reads three arguments and leaves volatile registers changed.
    let callee = assemble_at(CALLEE, |a| {
        a.setp(r11b)?; a.lea(rax, qword_ptr(rcx + rdx))?; a.add(rax, r8)?;
        a.mov(r9, rax)?; a.xor(r10d, r10d)?; a.ret()
    });
    let mut out = String::from(commons_recomp_emit::PRELUDE);
    out += "pub struct Case { pub name: &'static str, pub flags: u64, pub code: &'static [u8], pub translated: unsafe fn(&mut Ctx) }\n";
    out += &format!(
        "pub const CALLEE: &[u8] = &{callee:?};\npub const CODE_RVA: u64 = {IP:#x};\npub const CALLEE_RVA: u64 = {CALLEE:#x};\n"
    );
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
