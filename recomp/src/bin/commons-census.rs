//! `commons-census <exe> [--json OUT]`: what track B can take from a build, counted on your own copy.
//! Reads the executable, never runs it; prints counts only.

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(exe) = args.first() else {
        eprintln!("usage: commons-census <exe> [--json OUT]");
        return ExitCode::from(2);
    };
    let json = args.iter().position(|a| a == "--json").and_then(|i| args.get(i + 1));
    let data = match std::fs::read(exe) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("error: {exe}: {e}");
            return ExitCode::from(1);
        }
    };
    let image = match commons_recomp::census::Image::parse(&data) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("error: {exe}: {e}");
            return ExitCode::from(1);
        }
    };
    let start = std::time::Instant::now();
    let c = commons_recomp::census::census(&image, 25);
    println!("functions            {}", c.functions);
    println!("code bytes           {}", c.code_bytes);
    println!("instructions         {}", c.instructions);
    println!("covered              {:.2}% of instructions", c.covered_percent());
    println!("fully covered        {} functions ({:.1}%)", c.fully_covered_functions,
             100.0 * c.fully_covered_functions as f64 / c.functions.max(1) as f64);
    println!("  of them leaves     {}", c.fully_covered_leaves);
    println!("decode errors        {}", c.decode_errors);
    if c.unreadable_functions > 0 {
        println!("not in the file      {} functions", c.unreadable_functions);
    }
    println!("most often uncovered:");
    for (m, n) in &c.top_uncovered {
        println!("  {m:<20} {n}");
    }
    println!("({:.1} s)", start.elapsed().as_secs_f64());
    if let Some(out) = json {
        match serde_json::to_string_pretty(&c) {
            Ok(text) => {
                if let Err(e) = std::fs::write(out, text) {
                    eprintln!("error: {out}: {e}");
                    return ExitCode::from(1);
                }
            }
            Err(e) => {
                eprintln!("error: {e}");
                return ExitCode::from(1);
            }
        }
    }
    ExitCode::SUCCESS
}
