//! `commons-recomp <exe> --work DIR [--limit N] [--rounds N] [--jobs N]`: track B for this build, on this machine.
//! Selects checkable functions, generates and builds the library in DIR, tests every function against the native
//! one, and writes DIR/<build>.toml for `cargo commons run` (with COMMONS_RECOMP_DIR=DIR). Nothing is written into
//! the repository. The differential test runs in `--jobs` processes side by side (default: half the cores), each
//! on its share of the functions (`--part K/N`, with `--library` the built one); a process that dies loses only
//! its share.

use std::path::PathBuf;
use std::process::ExitCode;

fn arg(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned()
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (Some(exe), Some(work)) = (args.first().filter(|a| !a.starts_with("--")), arg(&args, "--work")) else {
        eprintln!("usage: commons-recomp <exe> --work DIR [--limit N] [--rounds N]");
        return ExitCode::from(2);
    };
    let limit = arg(&args, "--limit").and_then(|v| v.parse().ok()).unwrap_or(5000usize);
    let rounds = arg(&args, "--rounds").and_then(|v| v.parse().ok()).unwrap_or(300usize);
    let cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(2);
    let jobs = arg(&args, "--jobs").and_then(|v| v.parse().ok()).unwrap_or((cores / 2).max(1)).max(1);
    let part: Option<(usize, usize)> = arg(&args, "--part").and_then(|v| {
        let (k, n) = v.split_once('/')?;
        Some((k.parse().ok()?, n.parse().ok()?))
    });
    let work = PathBuf::from(work);
    let data = match std::fs::read(exe) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("error: {exe}: {e}");
            return ExitCode::from(1);
        }
    };
    let build = blake3::hash(&data).to_hex().to_string();
    let image = match commons_recomp::census::Image::parse(&data) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("error: {exe}: {e}");
            return ExitCode::from(1);
        }
    };
    if part.is_none() {
        println!("build          {build}");
    }
    let callees = commons_recomp::pipeline::CalleeCheck::new(&image);
    let rule = |code: &[u8], rva: u32| {
        commons_recomp::pipeline::checkable_with_calls(code, rva, &|t| callees.safe(t, 4))
    };
    let cands = commons_recomp::pipeline::select(&image, limit, &rule);
    if let Some((k, n)) = part {
        // One share of the differential test, for the process that started this one.
        let Some(dll) = arg(&args, "--library") else {
            eprintln!("error: --part needs --library");
            return ExitCode::from(2);
        };
        let mine: Vec<_> = cands.into_iter().enumerate().filter(|(i, _)| i % n == k).map(|(_, c)| c).collect();
        let lib = match commons_recomp::pipeline::load(std::path::Path::new(&dll)) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("error: {e}");
                return ExitCode::from(1);
            }
        };
        let base = match commons_recomp::pipeline::map_image(&image) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("error: {e}");
                return ExitCode::from(1);
            }
        };
        let report = commons_recomp::pipeline::difftest_sandboxed(&mine, &lib, base, image.size_of_image as usize, rounds);
        use std::io::Write as _;
        let mut out = std::io::stdout().lock();
        for rva in &report.passed {
            let _ = writeln!(out, "pass {rva:#x}");
        }
        for rva in &report.differed_rvas {
            let _ = writeln!(out, "differed {rva:#x}");
        }
        let _ = writeln!(out, "inconclusive {}", report.inconclusive);
        let _ = out.flush();
        terminate();
    }
    println!("selected       {} functions (memory through registers and globals; loops under a watchdog; calls to safe callees)", cands.len());
    let lib_rs = match commons_recomp::pipeline::generate(&cands) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: generating: {e}");
            return ExitCode::from(1);
        }
    };
    println!("building       {}", work.join("gen").display());
    let dll = match commons_recomp::pipeline::build(&work, &lib_rs) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(1);
        }
    };
    let lib = match commons_recomp::pipeline::load(&dll) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(1);
        }
    };
    drop(lib);
    println!("difftest       in {jobs} processes");
    let me = std::env::current_exe().expect("own path");
    let children: Vec<_> = (0..jobs)
        .map(|k| {
            std::process::Command::new(&me)
                .args([exe.as_str(), "--work", &work.display().to_string(), "--limit", &limit.to_string()])
                .args(["--rounds", &rounds.to_string(), "--part", &format!("{k}/{jobs}"), "--library", &dll.display().to_string()])
                .stdout(std::process::Stdio::piped())
                .spawn()
        })
        .collect();
    let mut report = commons_recomp::pipeline::SandboxReport { passed: Vec::new(), differed: 0, differed_rvas: Vec::new(), inconclusive: 0 };
    let mut lost = 0usize;
    for (k, child) in children.into_iter().enumerate() {
        let out = child.and_then(|c| c.wait_with_output());
        let ok = matches!(&out, Ok(o) if o.status.success());
        let text = out.map(|o| String::from_utf8_lossy(&o.stdout).into_owned()).unwrap_or_default();
        if !ok || !text.contains("inconclusive ") {
            println!("lost           share {k} of {jobs} (its process failed); its functions stay native");
            lost += 1;
            continue;
        }
        for line in text.lines() {
            let mut w = line.split_whitespace();
            let (Some(kind), Some(v)) = (w.next(), w.next()) else { continue };
            let rva = u32::from_str_radix(v.trim_start_matches("0x"), 16).ok();
            match (kind, rva) {
                ("pass", Some(r)) => report.passed.push(r),
                ("differed", Some(r)) => {
                    report.differed += 1;
                    report.differed_rvas.push(r);
                }
                ("inconclusive", _) => report.inconclusive += v.parse::<usize>().unwrap_or(0),
                _ => {}
            }
        }
    }
    let _ = lost;
    report.passed.sort_unstable();
    report.differed_rvas.sort_unstable();
    let passed = report.passed;
    for rva in report.differed_rvas.iter().take(20) {
        println!("differed       {rva:#x}");
    }
    println!(
        "difftest       {} equal to the native function, {} differed, {} inconclusive (of {}; {rounds} inputs each)",
        passed.len(), report.differed, report.inconclusive, cands.len()
    );
    let manifest = commons_recomp::pipeline::manifest(&build, &dll, &cands, &passed);
    let path = work.join(format!("{build}.toml"));
    if let Err(e) = std::fs::write(&path, manifest) {
        eprintln!("error: {}: {e}", path.display());
        return ExitCode::from(1);
    }
    println!("manifest       {}", path.display());
    println!("run            set COMMONS_RECOMP_DIR={} and cargo commons run <exe> --execute", work.display());
    terminate()
}

/// The process holds the mapped image, the loaded library and the watchdog thread; tearing those down at exit has
/// crashed after the work was done. Everything is written: end here.
fn terminate() -> ! {
    use std::io::Write as _;
    let _ = std::io::stdout().flush();
    unsafe extern "system" {
        fn GetCurrentProcess() -> isize;
        fn TerminateProcess(process: isize, code: u32) -> i32;
    }
    unsafe { TerminateProcess(GetCurrentProcess(), 0) };
    unreachable!()
}
