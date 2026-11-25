//! The emitter checked against the CPU: every synthetic function of build.rs runs natively (through a small thunk
//! that loads all sixteen registers from a context) and translated, on the same random inputs, memory and stack,
//! and the registers, the compared flags, the memory and the stack must agree.
#![cfg(all(windows, target_arch = "x86_64"))]

include!(concat!(env!("OUT_DIR"), "/synthetic.rs"));

use std::arch::asm;


static mut HOST_RSP: u64 = 0;
static mut CTX: *mut Ctx = core::ptr::null_mut();
static mut TARGET: u64 = 0;
static mut SCRATCH: u64 = 0;
static mut FLAGS: u64 = 0;

/// Calls native code at `code` with every register from `c` (rsp included) and stores them back afterwards.
/// Returns RFLAGS as the code left them. The ymm registers travel too.
#[target_feature(enable = "avx")]
unsafe fn run_native(code: u64, c: &mut Ctx) -> u64 {
    unsafe {
        CTX = c as *mut Ctx;
        TARGET = code;
        asm!(
            "push rbx",
            "push rbp",
            "mov [rip + {host}], rsp",
            "mov rax, [rip + {ctx}]",
            "mov rsp, [rax + 32]",
            "mov rcx, [rax + 8]",
            "mov rdx, [rax + 16]",
            "mov rbx, [rax + 24]",
            "mov rbp, [rax + 40]",
            "mov rsi, [rax + 48]",
            "mov rdi, [rax + 56]",
            "mov r8, [rax + 64]",
            "mov r9, [rax + 72]",
            "mov r10, [rax + 80]",
            "mov r11, [rax + 88]",
            "mov r12, [rax + 96]",
            "mov r13, [rax + 104]",
            "mov r14, [rax + 112]",
            "mov r15, [rax + 120]",
            "vmovdqu ymm0, [rax + 160]",
            "vmovdqu ymm1, [rax + 192]",
            "vmovdqu ymm2, [rax + 224]",
            "vmovdqu ymm3, [rax + 256]",
            "vmovdqu ymm4, [rax + 288]",
            "vmovdqu ymm5, [rax + 320]",
            "vmovdqu ymm6, [rax + 352]",
            "vmovdqu ymm7, [rax + 384]",
            "vmovdqu ymm8, [rax + 416]",
            "vmovdqu ymm9, [rax + 448]",
            "vmovdqu ymm10, [rax + 480]",
            "vmovdqu ymm11, [rax + 512]",
            "vmovdqu ymm12, [rax + 544]",
            "vmovdqu ymm13, [rax + 576]",
            "vmovdqu ymm14, [rax + 608]",
            "vmovdqu ymm15, [rax + 640]",
            "mov rax, [rax]",
            "call qword ptr [rip + {target}]",
            "mov [rip + {scratch}], rax",
            "pushfq",
            "pop rax",
            "mov [rip + {flags}], rax",
            "mov rax, [rip + {ctx}]",
            "mov [rax + 8], rcx",
            "mov [rax + 16], rdx",
            "mov [rax + 24], rbx",
            "mov [rax + 32], rsp",
            "mov [rax + 40], rbp",
            "mov [rax + 48], rsi",
            "mov [rax + 56], rdi",
            "mov [rax + 64], r8",
            "mov [rax + 72], r9",
            "mov [rax + 80], r10",
            "mov [rax + 88], r11",
            "mov [rax + 96], r12",
            "mov [rax + 104], r13",
            "mov [rax + 112], r14",
            "mov [rax + 120], r15",
            "vmovdqu [rax + 160], ymm0",
            "vmovdqu [rax + 192], ymm1",
            "vmovdqu [rax + 224], ymm2",
            "vmovdqu [rax + 256], ymm3",
            "vmovdqu [rax + 288], ymm4",
            "vmovdqu [rax + 320], ymm5",
            "vmovdqu [rax + 352], ymm6",
            "vmovdqu [rax + 384], ymm7",
            "vmovdqu [rax + 416], ymm8",
            "vmovdqu [rax + 448], ymm9",
            "vmovdqu [rax + 480], ymm10",
            "vmovdqu [rax + 512], ymm11",
            "vmovdqu [rax + 544], ymm12",
            "vmovdqu [rax + 576], ymm13",
            "vmovdqu [rax + 608], ymm14",
            "vmovdqu [rax + 640], ymm15",
            "mov rcx, [rip + {scratch}]",
            "mov [rax], rcx",
            "mov rsp, [rip + {host}]",
            "pop rbp",
            "pop rbx",
            host = sym HOST_RSP,
            ctx = sym CTX,
            target = sym TARGET,
            scratch = sym SCRATCH,
            flags = sym FLAGS,
            out("rax") _, out("rcx") _, out("rdx") _, out("rsi") _, out("rdi") _,
            out("r8") _, out("r9") _, out("r10") _, out("r11") _,
            out("r12") _, out("r13") _, out("r14") _, out("r15") _,
            out("ymm0") _, out("ymm1") _, out("ymm2") _, out("ymm3") _, out("ymm4") _, out("ymm5") _,
            out("ymm6") _, out("ymm7") _, out("ymm8") _, out("ymm9") _, out("ymm10") _, out("ymm11") _,
            out("ymm12") _, out("ymm13") _, out("ymm14") _, out("ymm15") _,
        );
        FLAGS
    }
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    /// Two floats or one double: ordinary values, zeros, infinities, NaNs, denormals and random bits.
    fn vector(&mut self) -> u64 {
        let f = |r: &mut Rng| -> u32 {
            match r.next() % 8 {
                0 => 0,
                1 => 0x8000_0000,
                2 => 0x7f80_0000,
                3 => 0x7fc0_0001,
                4 => 0x0000_0001,
                5 => ((r.next() % 2000) as f32 - 1000.0).to_bits(),
                6 => (((r.next() % 1_000_000) as f32) / 997.0).to_bits(),
                _ => r.next() as u32,
            }
        };
        if self.next() % 4 == 0 {
            ((self.next() % 4000) as f64 / 3.0 - 600.0).to_bits()
        } else {
            u64::from(f(self)) | (u64::from(f(self)) << 32)
        }
    }
    /// Mostly random, sometimes the edge values that flags care about.
    fn value(&mut self) -> u64 {
        match self.next() % 8 {
            0 => 0,
            1 => u64::MAX,
            2 => 1u64 << 63,
            3 => 0x7fff_ffff,
            4 => 0x8000_0000,
            5 => self.next() & 0xff,
            _ => self.next(),
        }
    }
}

/// run_native and the TLS setup use process-wide state: one test at a time.
static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

const MEM: usize = 1024;
const STACK: usize = 4096;

#[test]
fn translated_functions_match_the_cpu() {
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let exec = unsafe { VirtualAlloc(core::ptr::null_mut(), 1 << 16, 0x3000, 0x40) } as *mut u8;
    assert!(!exec.is_null());
    let mut memory = vec![0u8; MEM];
    let mut stack = vec![0u8; STACK];
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    // The exec block is the "image": cases at CODE_RVA, the native callee of the call cases at CALLEE_RVA.
    unsafe {
        set_image_base(exec as u64);
        setup();
        std::ptr::copy_nonoverlapping(CALLEE.as_ptr(), exec.add(CALLEE_RVA as usize), CALLEE.len());
    }
    let entry = exec as u64 + CODE_RVA;
    for case in CASES {
        unsafe { std::ptr::copy_nonoverlapping(case.code.as_ptr(), entry as *mut u8, case.code.len()) };
        for round in 0..3000 {
            let mut start = Ctx::default();
            for r in 0..16 {
                start.r[r] = rng.value();
                for q in 0..4 {
                    start.x[r][q] = rng.vector();
                }
            }
            let initial_memory: Vec<u8> = (0..MEM).map(|_| rng.next() as u8).collect();
            let base = memory.as_mut_ptr() as u64;
            start.r[2] = base + 64 + (rng.next() % 64); // rdx points into the buffer
            if case.name == "call_indirect" {
                start.r[15] = exec as u64 + CALLEE_RVA;
            }
            if case.name == "loop" {
                start.r[1] = rng.next() % 32; // rcx: how many quadwords to add
                start.r[2] = base + 64;
            }
            let top = ((stack.as_mut_ptr() as u64 + STACK as u64) & !0xf) - 64;

            // Native: `call` pushes the return address at top - 8.
            memory.copy_from_slice(&initial_memory);
            stack.fill(0);
            let mut native = start;
            native.r[4] = top;
            let rflags = unsafe { run_native(entry, &mut native) };
            let native_memory = memory.clone();
            let mut native_stack = stack.clone();

            // Translated: entered with the return address already on the stack, as the thunk will.
            memory.copy_from_slice(&initial_memory);
            stack.fill(0);
            let mut ours = start;
            ours.r[4] = top - 8;
            unsafe { ((top - 8) as *mut u64).write(0x5151_5151) };
            unsafe { (case.translated)(&mut ours) };

            let slot = (top - 8 - stack.as_ptr() as u64) as usize;
            let mut our_stack = stack.clone();
            for s in [&mut native_stack, &mut our_stack] {
                s[slot..slot + 8].fill(0); // the return address (and the thunk's pushfq) differ by design
                if case.name.starts_with("call") {
                    // Below the caller's frame: the inner call's return address and the thunk's scratch words.
                    s[slot - 32..slot - 8].fill(0);
                }
            }
            let flag = |bit: u64| rflags & bit != 0;
            let want = |bit: u64| case.flags & bit != 0;
            let ctx = format!("{} round {round}: start {:x?}", case.name, start.r);
            assert_eq!(ours.r, native.r, "registers, {ctx}");
            assert_eq!(ours.x, native.x, "ymm registers, {ctx}");
            assert_eq!(memory, native_memory, "memory, {ctx}");
            assert_eq!(our_stack, native_stack, "stack, {ctx}");
            if want(0x1) { assert_eq!(ours.cf, flag(0x1), "CF, {ctx}"); }
            if want(0x40) { assert_eq!(ours.zf, flag(0x40), "ZF, {ctx}"); }
            if want(0x80) { assert_eq!(ours.sf, flag(0x80), "SF, {ctx}"); }
            if want(0x800) { assert_eq!(ours.of, flag(0x800), "OF, {ctx}"); }
            if want(0x4) { assert_eq!(ours.pf, flag(0x4), "PF, {ctx}"); }
            assert_eq!(ours.ret, 0x5151_5151, "return address, {ctx}");
        }
    }
}

/// Native code calls two translations through their patched entries, one nested inside a native callee of the
/// other: the title's results must not change. Run once unpatched and once patched on the same inputs.
#[test]
fn patched_entries_keep_the_results() {
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let exec = unsafe { VirtualAlloc(core::ptr::null_mut(), 1 << 16, 0x3000, 0x40) } as *mut u8;
    assert!(!exec.is_null());
    let place = |rva: usize, code: &[u8]| unsafe { std::ptr::copy_nonoverlapping(code.as_ptr(), exec.add(rva), code.len()) };
    let original = || {
        place(0x3000, ENTRY_F);
        place(0x4000, ENTRY_G);
        place(0x5000, ENTRY_H);
        place(0x6000, ENTRY_CALLER);
    };
    let jump = |rva: usize, to: u64| {
        let mut j = [0u8; 14];
        j[..6].copy_from_slice(&[0xFF, 0x25, 0, 0, 0, 0]);
        j[6..].copy_from_slice(&to.to_le_bytes());
        place(rva, &j);
    };
    unsafe {
        set_image_base(exec as u64);
        setup();
    }
    let mut stack = vec![0u8; 1 << 16];
    let top = ((stack.as_mut_ptr() as u64 + stack.len() as u64) & !0xf) - 64;
    let mut rng = Rng(0x2545_f491_4f6c_dd1d);
    for round in 0..2000 {
        let mut start = Ctx::default();
        for r in 0..16 {
            start.r[r] = rng.value();
            for q in 0..4 {
                start.x[r][q] = rng.vector();
            }
        }
        start.r[4] = top;

        original();
        let mut native = start;
        unsafe { run_native(exec as u64 + 0x6000, &mut native) };

        original();
        jump(0x3000, e_entry_f as usize as u64);
        jump(0x5000, e_entry_h as usize as u64);
        let mut patched = start;
        unsafe { run_native(exec as u64 + 0x6000, &mut patched) };

        let ctx = format!("round {round}: start {:x?}", start.r);
        assert_eq!(patched.r, native.r, "registers, {ctx}");
        assert_eq!(patched.x, native.x, "ymm registers, {ctx}");
    }
}
