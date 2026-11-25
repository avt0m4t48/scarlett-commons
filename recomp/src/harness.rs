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
    pub pf: bool,
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
/// Fault recovery: while a call through the thunk runs, a fault in the called code resumes at the thunk's exit.
static mut IN_RUN: bool = false;
static mut FAULTED: bool = false;
static mut RECOVER: u64 = 0;
static VEH: std::sync::Once = std::sync::Once::new();

unsafe extern "system" {
    fn AddVectoredExceptionHandler(first: u32, handler: unsafe extern "system" fn(*mut u8) -> i32) -> *mut u8;
}

/// EXCEPTION_POINTERS -> (ExceptionRecord, ContextRecord); the code is the record's first field and Rip sits at 0xF8 of
/// the x64 CONTEXT.
unsafe extern "system" fn on_fault(pointers: *mut u8) -> i32 {
    unsafe {
        let record = *(pointers as *const *const u32);
        let context = *(pointers.add(8) as *const *mut u8);
        let code = *record;
        let recoverable = matches!(code, 0xC000_0005 | 0xC000_001D | 0xC000_0094 | 0xC000_0096 | 0xC000_008C | 0x8000_0002);
        if IN_RUN && recoverable && RECOVER != 0 {
            FAULTED = true;
            *(context.add(0xF8) as *mut u64) = RECOVER;
            return -1; // EXCEPTION_CONTINUE_EXECUTION
        }
        0 // EXCEPTION_CONTINUE_SEARCH
    }
}

/// [`run_native`], but a fault in the called code (an access violation, an illegal or privileged instruction, a
/// division error) returns None instead of ending the process.
///
/// # Safety
/// As [`run_native`].
#[target_feature(enable = "avx")]
pub unsafe fn run_native_guarded(code: u64, c: &mut HostCtx) -> Option<u64> {
    VEH.call_once(|| unsafe {
        AddVectoredExceptionHandler(1, on_fault);
    });
    unsafe {
        FAULTED = false;
        let flags = run_native(code, c);
        if FAULTED { None } else { Some(flags) }
    }
}

/// How a timed call ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Done(u64),
    Fault,
    Timeout,
}

static DEADLINE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static TIMED_OUT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static WATCHDOG: std::sync::Once = std::sync::Once::new();
static mut RUN_THREAD: isize = 0;

unsafe extern "system" {
    fn GetCurrentThreadId() -> u32;
    fn OpenThread(access: u32, inherit: i32, id: u32) -> isize;
    fn SuspendThread(thread: isize) -> u32;
    fn ResumeThread(thread: isize) -> u32;
    fn GetThreadContext(thread: isize, context: *mut u8) -> i32;
    fn SetThreadContext(thread: isize, context: *const u8) -> i32;
}

#[repr(C, align(16))]
struct Context([u8; 1232]);

fn now_us() -> u64 {
    static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    START.get_or_init(std::time::Instant::now).elapsed().as_micros() as u64
}

/// The watchdog: when a timed call is past its deadline and still inside the thunk's call, the calling thread is
/// suspended, its rip moved to the thunk's exit, and resumed.
fn watchdog() {
    loop {
        std::thread::sleep(std::time::Duration::from_micros(200));
        let deadline = DEADLINE.load(std::sync::atomic::Ordering::Acquire);
        if deadline == 0 || now_us() < deadline {
            continue;
        }
        unsafe {
            let t = RUN_THREAD;
            if SuspendThread(t) == u32::MAX {
                continue;
            }
            let inside = core::ptr::read_volatile(&raw const IN_RUN);
            if inside && DEADLINE.load(std::sync::atomic::Ordering::Acquire) == deadline {
                let mut ctx = Context([0; 1232]);
                *(ctx.0.as_mut_ptr().add(0x30) as *mut u32) = 0x0010_0001; // CONTEXT_CONTROL
                if GetThreadContext(t, ctx.0.as_mut_ptr()) != 0 {
                    *(ctx.0.as_mut_ptr().add(0xF8) as *mut u64) = RECOVER;
                    if SetThreadContext(t, ctx.0.as_ptr()) != 0 {
                        TIMED_OUT.store(true, std::sync::atomic::Ordering::Release);
                        DEADLINE.store(0, std::sync::atomic::Ordering::Release);
                    }
                }
            }
            ResumeThread(t);
        }
    }
}

/// [`run_native_guarded`] with a time limit: a call still running after `limit_us` microseconds is stopped and reported
/// as a timeout (a loop on a generated count, say).
///
/// # Safety
/// As [`run_native`].
#[target_feature(enable = "avx")]
pub unsafe fn run_native_timed(code: u64, c: &mut HostCtx, limit_us: u64) -> Outcome {
    unsafe {
        WATCHDOG.call_once(|| {
            RUN_THREAD = OpenThread(0x001F_03FF, 0, GetCurrentThreadId());
            std::thread::spawn(watchdog);
        });
        TIMED_OUT.store(false, std::sync::atomic::Ordering::Release);
        DEADLINE.store(now_us() + limit_us, std::sync::atomic::Ordering::Release);
        let r = run_native_guarded(code, c);
        DEADLINE.store(0, std::sync::atomic::Ordering::Release);
        if TIMED_OUT.load(std::sync::atomic::Ordering::Acquire) {
            Outcome::Timeout
        } else {
            match r {
                Some(f) => Outcome::Done(f),
                None => Outcome::Fault,
            }
        }
    }
}

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
            "lea rax, [rip + 9f]",
            "mov [rip + {recover}], rax",
            "mov byte ptr [rip + {inrun}], 1",
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
            "mov byte ptr [rip + {inrun}], 0",
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
            "9:",
            "mov byte ptr [rip + {inrun}], 0",
            "mov rsp, [rip + {host}]",
            "pop rbp",
            "pop rbx",
            host = sym HOST_RSP,
            ctx = sym CTX,
            target = sym TARGET,
            scratch = sym SCRATCH,
            flags = sym FLAGS,
            recover = sym RECOVER,
            inrun = sym IN_RUN,
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

#[cfg(test)]
mod tests {
    use super::*;

    unsafe extern "system" {
        fn VirtualAlloc(address: *mut u8, size: usize, kind: u32, protect: u32) -> *mut u8;
    }

    #[test]
    fn faults_and_endless_loops_come_back() {
        let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let exec = unsafe { VirtualAlloc(core::ptr::null_mut(), 4096, 0x3000, 0x40) };
        let code: [&[u8]; 3] = [
            &[0x48, 0x89, 0xC8, 0xC3],             // mov rax, rcx; ret
            &[0x48, 0x8B, 0x00, 0xC3],             // mov rax, [rax]; ret (rax = 0: faults)
            &[0xEB, 0xFE],                         // jmp $: never returns
        ];
        for (k, c) in code.iter().enumerate() {
            unsafe { std::ptr::copy_nonoverlapping(c.as_ptr(), exec.add(64 * k), c.len()) };
        }
        let mut stack = vec![0u8; 1 << 16];
        let top = (stack.as_mut_ptr() as u64 + (1 << 15)) & !0xf;
        let run = |k: usize, limit: u64| {
            let mut c = HostCtx::default();
            c.r[1] = 0x1234;
            c.r[4] = top;
            let outcome = unsafe { run_native_timed(exec as u64 + 64 * k as u64, &mut c, limit) };
            (outcome, c.r[0])
        };
        assert!(matches!(run(0, 2_000), (Outcome::Done(_), 0x1234)));
        assert_eq!(run(1, 2_000).0, Outcome::Fault);
        assert_eq!(run(2, 2_000).0, Outcome::Timeout);
        assert!(matches!(run(0, 2_000), (Outcome::Done(_), 0x1234)), "a call after a timeout works");
    }
}
