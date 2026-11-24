//! The emitter checked against the CPU: every synthetic function of build.rs runs natively (through a small thunk
//! that loads all sixteen registers from a context) and translated, on the same random inputs, memory and stack,
//! and the registers, the compared flags, the memory and the stack must agree.
#![cfg(all(windows, target_arch = "x86_64"))]

include!(concat!(env!("OUT_DIR"), "/synthetic.rs"));

use std::arch::asm;

unsafe extern "system" {
    fn VirtualAlloc(address: *mut u8, size: usize, kind: u32, protect: u32) -> *mut u8;
}

static mut HOST_RSP: u64 = 0;
static mut CTX: *mut Ctx = core::ptr::null_mut();
static mut TARGET: u64 = 0;
static mut SCRATCH: u64 = 0;
static mut FLAGS: u64 = 0;

/// Calls native code at `code` with every register from `c` (rsp included) and stores them back afterwards.
/// Returns RFLAGS as the code left them.
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

const MEM: usize = 1024;
const STACK: usize = 4096;

#[test]
fn translated_functions_match_the_cpu() {
    let exec = unsafe { VirtualAlloc(core::ptr::null_mut(), 1 << 16, 0x3000, 0x40) };
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
            assert_eq!(memory, native_memory, "memory, {ctx}");
            assert_eq!(our_stack, native_stack, "stack, {ctx}");
            if want(0x1) { assert_eq!(ours.cf, flag(0x1), "CF, {ctx}"); }
            if want(0x40) { assert_eq!(ours.zf, flag(0x40), "ZF, {ctx}"); }
            if want(0x80) { assert_eq!(ours.sf, flag(0x80), "SF, {ctx}"); }
            if want(0x800) { assert_eq!(ours.of, flag(0x800), "OF, {ctx}"); }
            assert_eq!(ours.ret, 0x5151_5151, "return address, {ctx}");
        }
    }
}
