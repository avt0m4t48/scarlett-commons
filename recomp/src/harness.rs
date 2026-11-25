//! Running native code with any register state: the differential tester and the tests call a function natively
//! through this thunk, with every GPR (rsp included) and ymm register loaded from a context, and read back what it
//! left. Single-threaded: the thunk keeps its own state in statics, so callers hold [`LOCK`].
#![cfg(all(windows, target_arch = "x86_64"))]

use std::arch::asm;

/// The context the generated code uses (`Ctx` in commons-recomp-emit's prelude), with the same layout.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HostCtx {
    pub r: [u64; 16],
    pub cf: bool,
    pub zf: bool,
    pub sf: bool,
    pub of: bool,
    pub ret: u64,
    pub host_rsp: u64,
    pub target: u64,
    pub x: [[u64; 4]; 16],
}

pub static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

static mut HOST_RSP: u64 = 0;
static mut CTX: *mut HostCtx = core::ptr::null_mut();
static mut TARGET: u64 = 0;
static mut SCRATCH: u64 = 0;
static mut FLAGS: u64 = 0;

/// Calls native code at `code` with every register from `c` and stores them back afterwards; returns RFLAGS.
///
/// # Safety
/// `code` must be executable and `c.r[4]` a stack the code may use.
#[target_feature(enable = "avx")]
pub unsafe fn run_native(code: u64, c: &mut HostCtx) -> u64 {
    unsafe {
        CTX = c as *mut HostCtx;
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

/// A small xorshift generator with the edge values flags and floats care about.
pub struct Rng(pub u64);

impl Rng {
    pub fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    pub fn value(&mut self) -> u64 {
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

    pub fn vector(&mut self) -> u64 {
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
}
