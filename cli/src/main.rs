//! uconv — 単位と SI 接頭辞の CLI。
//!
//! 変換の処理はすべて `unitconv-core` 側にある（同じコアを C# GUI が cdylib 経由で使う）。
//! このファイルは引数を解釈して表示するだけ。
//!
//!   uconv 5 km mile          単位変換（接頭辞つき）
//!   uconv 3.2 GHz MHz
//!   uconv scale 5 m p        接頭辞スケーリング（5 ミリ → ピコ = 5e9）
//!   uconv base ff 16         進数変換
//!   uconv --prefixes         接頭辞の一覧
//!   uconv --units            単位の一覧
//!   uconv                    引数なしで対話モード

use std::io::{self, BufRead, Write};

use unitconv_core as core;

fn print_help() {
    println!(
        "uconv — SI接頭辞と単位を爆速で変換する\n\
\n\
  使い方:\n\
    uconv <値> <変換前> <変換後>          単位変換（接頭辞つき）\n\
    uconv scale <値> <接頭辞A> <接頭辞B>  接頭辞スケーリング\n\
    uconv base <値> <基数>                2/8/10/16 進数へ\n\
    uconv --prefixes                     接頭辞の一覧\n\
    uconv --units                        単位の一覧\n\
    uconv                                対話モード\n\
\n\
  例:\n\
    uconv 5 km mile           → 3.1068559612 mile\n\
    uconv 3.2 GHz MHz         → 3200 MHz\n\
    uconv 0.0000035 m mm      → 0.0035 mm  (≈ 3.5 µm)\n\
    uconv 100 degC degF       → 212 °F\n\
    uconv scale 5 m p         → 5000000000\n\
    uconv 1 GiB MB            → 1073.741824 MB\n\
\n\
  温度は °C / degC / °F / degF / K（C はクーロン、F はファラドなのに注意）"
    );
}

fn print_prefixes() {
    println!("接頭辞（大きい順。Ki〜Ti は 1024^n の2進接頭辞）\n");
    println!("{:<10} {:<4} {}", "名前", "記号", "倍率");
    for prefix in core::prefix_table() {
        println!("{:<10} {:<4} {:e}", prefix.name, prefix.symbol, prefix.mult);
    }
}

fn print_units() {
    let all = core::units();
    let mut groups: Vec<&'static str> = Vec::new();
    for unit in &all {
        if !groups.contains(&unit.group) {
            groups.push(unit.group);
        }
    }
    for group in groups {
        println!("\n{group}:");
        for unit in all.iter().filter(|unit| unit.group == group) {
            let extra = if unit.prefixable { " 接頭辞OK" } else { "" };
            let aliases = if unit.aliases.is_empty() {
                String::new()
            } else {
                format!("  ({})", unit.aliases.join(", "))
            };
            println!("  {:<5}{extra}{aliases}", unit.name);
        }
    }
}

/// 変換結果を 1 つの文字列（pretty があれば 2 行目）にする。
fn format_convert(result: core::ConvertResult) -> Result<String, String> {
    if !result.ok {
        return Err(result.error.unwrap_or_else(|| "変換できません".to_string()));
    }
    let mut text = format!("{} {}", core::fmt_num(result.value.unwrap_or(0.0)), result.unit.unwrap_or_default());
    if let Some(pretty) = result.pretty {
        text.push_str(&format!("\n≈ {pretty}"));
    }
    Ok(text)
}

fn format_scale(result: core::ConvertResult) -> Result<String, String> {
    if !result.ok {
        return Err(result.error.unwrap_or_else(|| "変換できません".to_string()));
    }
    Ok(core::fmt_num(result.value.unwrap_or(0.0)))
}

fn repl() {
    println!("uconv 対話モード。例: 5 km mile / scale 5 m p / help / quit");
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    loop {
        print!("> ");
        let _ = stdout.flush();
        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => break,
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if matches!(line, "quit" | "exit" | ":q" | "q") {
            break;
        }
        if matches!(line, "help" | "?" | ":h") {
            print_help();
            continue;
        }
        if line == "prefixes" {
            print_prefixes();
            continue;
        }
        if line == "units" {
            print_units();
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        let result = match parts.as_slice() {
            ["scale", v, a, b] => format_scale(core::scale(v, a, b)),
            ["base", v, base] => core::base(v, base),
            [v, from, to] => format_convert(core::convert(v, from, to)),
            _ => Err("書式は「値 変換前 変換後」（または help）".to_string()),
        };
        match result {
            Ok(text) => println!("{text}"),
            Err(message) => println!("error: {message}"),
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        repl();
        return;
    }

    let command = args[0].as_str();
    let outcome: Result<(), String> = match command {
        "-h" | "--help" | "help" | "?" => {
            print_help();
            Ok(())
        }
        "--prefixes" | "-p" | "prefixes" => {
            print_prefixes();
            Ok(())
        }
        "--units" | "-u" | "units" => {
            print_units();
            Ok(())
        }
        "scale" => match args.as_slice() {
            [_, v, a, b] => format_scale(core::scale(v, a, b)).map(|text| println!("{text}")),
            _ => Err("使い方: uconv scale <値> <接頭辞A> <接頭辞B>".to_string()),
        },
        "base" => match args.as_slice() {
            [_, v, base] => core::base(v, base).map(|text| println!("{text}")),
            _ => Err("使い方: uconv base <値> <基数>".to_string()),
        },
        _ => match args.as_slice() {
            [v, from, to] => format_convert(core::convert(v, from, to)).map(|text| println!("{text}")),
            _ => Err("使い方: uconv <値> <変換前> <変換後>（引数なしで対話モード）".to_string()),
        },
    };

    if let Err(message) = outcome {
        eprintln!("error: {message}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_conversion() {
        let text = format_convert(core::convert("5", "km", "mile")).unwrap();
        assert!(text.starts_with("3.1068559612 mile"));
    }

    #[test]
    fn formats_scale() {
        assert_eq!(format_scale(core::scale("5", "m", "p")).unwrap(), "5e9");
    }

    #[test]
    fn propagates_error() {
        assert!(format_convert(core::convert("1", "km", "lb")).is_err());
    }
}
