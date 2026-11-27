//! `commons-ps5-scout <eboot.bin|.prx|.sprx>...`: what a PS5 executable of the user's own copy asks of the system
//! (modules, libraries, how many symbols each), and its functions and how much of their code track B can
//! translate today. Prints counts only; writes nothing.

use std::process::ExitCode;

fn main() -> ExitCode {
    let paths: Vec<String> = std::env::args().skip(1).collect();
    if paths.is_empty() {
        eprintln!("usage: commons-ps5-scout <eboot.bin|.prx|.sprx>...");
        return ExitCode::from(2);
    }
    let mut failed = false;
    for path in &paths {
        println!("== {path}");
        let data = match std::fs::read(path) {
            Ok(d) => d,
            Err(e) => {
                println!("error          {e}");
                failed = true;
                continue;
            }
        };
        match commons_recomp::ps5::imports(&data) {
            Ok(i) => {
                println!("modules        {}", i.modules.values().cloned().collect::<Vec<_>>().join(", "));
                let total: usize = i.symbols.values().map(Vec::len).sum();
                println!("imports        {total} symbols from {} libraries", i.libraries.len());
                let mut libs: Vec<_> = i.symbols.iter().map(|(id, s)| (s.len(), i.libraries.get(id).cloned().unwrap_or(format!("#{id}")))).collect();
                libs.sort_by(|a, b| b.0.cmp(&a.0));
                for (n, name) in libs {
                    println!("  {n:6}  {name}");
                }
                if !i.needed.is_empty() {
                    println!("needed         {}", i.needed.join(", "));
                }
            }
            Err(e) => {
                println!("imports        {e}");
                failed = true;
                continue;
            }
        }
        match commons_recomp::census::Image::parse(&data) {
            Ok(image) => {
                let c = commons_recomp::census::census(&image, 15);
                println!("functions      {} (from the unwind data)", image.functions().len());
                println!("{}", serde_json::to_string_pretty(&c).unwrap_or_default());
            }
            Err(e) => println!("census         {e}"),
        }
    }
    if failed { ExitCode::from(1) } else { ExitCode::SUCCESS }
}
