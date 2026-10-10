//! uconv — SI接頭辞と単位を爆速で変換する小さな CLI。
//!
//! 方針:
//! * 外部クレートなし（起動は一瞬、ネットもビルドも軽い）
//! * 「m（ミリ）」「p（ピコ）」のような 10^n の接頭辞をそのまま扱う
//! * 次元（長さ・質量・時間…）を検査して、おかしな変換は弾く
//! * 温度（°C / °F / K）はアフィン変換として別扱い
//! * 結果は「読みやすい接頭辞」でも併記する（例: 3.5 µm）
//!
//! 使い方:
//!   uconv 5 km mile          単位変換（接頭辞つき）
//!   uconv 3.2 GHz MHz
//!   uconv 0.0000035 m mm     すると「≈ 3.5 µm」も出る
//!   uconv 100 degC degF
//!   uconv scale 5 m p        接頭辞スケーリング（5 ミリ → ピコ = 5e9）
//!   uconv base ff 16         進数変換
//!   uconv --prefixes         接頭辞の一覧
//!   uconv --units            単位の一覧
//!   uconv                    引数なしで対話モード

use std::io::{self, BufRead, Write};

/// 次元の数。順に [m, kg, s, A, K, mol, cd, bit]。
const D: usize = 8;
type Dims = [i8; D];

const LEN: Dims = [1, 0, 0, 0, 0, 0, 0, 0];
const MASS: Dims = [0, 1, 0, 0, 0, 0, 0, 0];
const TIME: Dims = [0, 0, 1, 0, 0, 0, 0, 0];
const CURR: Dims = [0, 0, 0, 1, 0, 0, 0, 0];
const AMOUNT: Dims = [0, 0, 0, 0, 0, 1, 0, 0];
const LUM: Dims = [0, 0, 0, 0, 0, 0, 1, 0];
const INFO: Dims = [0, 0, 0, 0, 0, 0, 0, 1];

struct UnitDef {
    name: &'static str,
    aliases: &'static [&'static str],
    dims: Dims,
    /// 1 <単位> = factor [SI基本単位]
    factor: f64,
    /// SI接頭辞を付けられるか（m, g, s, Hz, B …）
    prefixable: bool,
    /// 一覧表示用のグループ名
    group: &'static str,
}

/// 接頭辞（記号）。2文字を先に試すため、その順で並べる。
const PREFIXES: &[(&str, f64)] = &[
    ("da", 1e1),
    ("Ki", 1024.0),
    ("Mi", 1_048_576.0),
    ("Gi", 1_073_741_824.0),
    ("Ti", 1_099_511_627_776.0),
    ("Pi", 1_125_899_906_842_624.0),
    ("Ei", 1.152_921_504_606_847e18),
    ("Y", 1e24),
    ("Z", 1e21),
    ("E", 1e18),
    ("P", 1e15),
    ("T", 1e12),
    ("G", 1e9),
    ("M", 1e6),
    ("k", 1e3),
    ("K", 1e3), // データで KB と書く慣習向け
    ("h", 1e2),
    ("d", 1e-1),
    ("c", 1e-2),
    ("m", 1e-3),
    ("µ", 1e-6),
    ("μ", 1e-6),
    ("u", 1e-6),
    ("n", 1e-9),
    ("p", 1e-12),
    ("f", 1e-15),
    ("a", 1e-18),
    ("z", 1e-21),
    ("y", 1e-24),
];

/// 接頭辞（名前）。`scale` と一覧で使う。
const PREFIX_NAMES: &[(&str, f64)] = &[
    ("yotta", 1e24),
    ("zetta", 1e21),
    ("exa", 1e18),
    ("peta", 1e15),
    ("tera", 1e12),
    ("giga", 1e9),
    ("mega", 1e6),
    ("kilo", 1e3),
    ("hecto", 1e2),
    ("deca", 1e1),
    ("deka", 1e1),
    ("deci", 1e-1),
    ("centi", 1e-2),
    ("milli", 1e-3),
    ("micro", 1e-6),
    ("nano", 1e-9),
    ("pico", 1e-12),
    ("femto", 1e-15),
    ("atto", 1e-18),
    ("zepto", 1e-21),
    ("yocto", 1e-24),
    ("kibi", 1024.0),
    ("mebi", 1_048_576.0),
    ("gibi", 1_073_741_824.0),
    ("tebi", 1_099_511_627_776.0),
];

fn u(
    name: &'static str,
    aliases: &'static [&'static str],
    dims: Dims,
    factor: f64,
    prefixable: bool,
    group: &'static str,
) -> UnitDef {
    UnitDef { name, aliases, dims, factor, prefixable, group }
}

fn units() -> Vec<UnitDef> {
    vec![
        // 長さ
        u("m", &["meter", "metre"], LEN, 1.0, true, "長さ"),
        u("mi", &["mile"], LEN, 1609.344, false, "長さ"),
        u("ft", &["foot", "feet"], LEN, 0.3048, false, "長さ"),
        u("in", &["inch"], LEN, 0.0254, false, "長さ"),
        u("yd", &["yard"], LEN, 0.9144, false, "長さ"),
        u("nmi", &["nauticalmile"], LEN, 1852.0, false, "長さ"),
        u("Å", &["angstrom"], LEN, 1e-10, false, "長さ"),
        u("au", &["astronomicalunit"], LEN, 1.495_978_707e11, false, "長さ"),
        u("ly", &["lightyear"], LEN, 9.460_730_472_580_8e15, false, "長さ"),
        u("pc", &["parsec"], LEN, 3.085_677_581_491_367_3e16, false, "長さ"),
        // 質量
        u("g", &["gram", "gramme"], MASS, 1e-3, true, "質量"),
        u("t", &["tonne", "ton"], MASS, 1e3, false, "質量"),
        u("lb", &["pound"], MASS, 0.453_592_37, false, "質量"),
        u("oz", &["ounce"], MASS, 0.028_349_523_125, false, "質量"),
        u("u", &["amu", "dalton", "Da"], MASS, 1.660_539_066_60e-27, false, "質量"),
        // 時間
        u("s", &["sec", "second"], TIME, 1.0, true, "時間"),
        u("min", &[], TIME, 60.0, false, "時間"),
        u("h", &["hr", "hour"], TIME, 3600.0, false, "時間"),
        u("day", &["d"], TIME, 86400.0, false, "時間"),
        u("wk", &["week"], TIME, 604_800.0, false, "時間"),
        u("yr", &["year"], TIME, 31_557_600.0, false, "時間"),
        // 電流・物量・光度
        u("A", &["ampere"], CURR, 1.0, true, "電気"),
        u("mol", &["mole"], AMOUNT, 1.0, true, "物量"),
        u("cd", &["candela"], LUM, 1.0, true, "光度"),
        // 振動数
        u("Hz", &["hertz"], [0, 0, -1, 0, 0, 0, 0, 0], 1.0, true, "振動数"),
        // 面積
        u("ha", &["hectare"], [2, 0, 0, 0, 0, 0, 0, 0], 1e4, false, "面積"),
        u("acre", &[], [2, 0, 0, 0, 0, 0, 0, 0], 4046.856_422_4, false, "面積"),
        // 体積
        u("L", &["l", "ltr", "liter", "litre"], [3, 0, 0, 0, 0, 0, 0, 0], 1e-3, true, "体積"),
        u("gal", &["gallon"], [3, 0, 0, 0, 0, 0, 0, 0], 0.003_785_411_784, false, "体積"),
        // 力・エネルギー・仕事率・圧力
        u("N", &["newton"], [1, 1, -2, 0, 0, 0, 0, 0], 1.0, true, "力"),
        u("J", &["joule"], [2, 1, -2, 0, 0, 0, 0, 0], 1.0, true, "エネルギー"),
        u("cal", &["calorie"], [2, 1, -2, 0, 0, 0, 0, 0], 4.184, false, "エネルギー"),
        u("kcal", &[], [2, 1, -2, 0, 0, 0, 0, 0], 4184.0, false, "エネルギー"),
        u("Wh", &[], [2, 1, -2, 0, 0, 0, 0, 0], 3600.0, true, "エネルギー"),
        u("eV", &["electronvolt"], [2, 1, -2, 0, 0, 0, 0, 0], 1.602_176_634e-19, true, "エネルギー"),
        u("W", &["watt"], [2, 1, -3, 0, 0, 0, 0, 0], 1.0, true, "仕事率"),
        u("hp", &["horsepower"], [2, 1, -3, 0, 0, 0, 0, 0], 745.699_871_582_270_2, false, "仕事率"),
        u("Pa", &["pascal"], [-1, 1, -2, 0, 0, 0, 0, 0], 1.0, true, "圧力"),
        u("bar", &[], [-1, 1, -2, 0, 0, 0, 0, 0], 1e5, false, "圧力"),
        u("atm", &["atmosphere"], [-1, 1, -2, 0, 0, 0, 0, 0], 101_325.0, false, "圧力"),
        u("mmHg", &[], [-1, 1, -2, 0, 0, 0, 0, 0], 133.322_387_415, false, "圧力"),
        u("psi", &[], [-1, 1, -2, 0, 0, 0, 0, 0], 6894.757_293_168, false, "圧力"),
        // 速さ・加速度
        u("kph", &["kmh"], [1, 0, -1, 0, 0, 0, 0, 0], 1.0 / 3.6, false, "速さ"),
        u("mph", &[], [1, 0, -1, 0, 0, 0, 0, 0], 0.44704, false, "速さ"),
        u("kn", &["knot"], [1, 0, -1, 0, 0, 0, 0, 0], 1852.0 / 3600.0, false, "速さ"),
        u("c", &["lightspeed"], [1, 0, -1, 0, 0, 0, 0, 0], 299_792_458.0, false, "速さ"),
        u("g0", &["gee"], [1, 0, -2, 0, 0, 0, 0, 0], 9.80665, false, "加速度"),
        // 電気・磁気
        u("C", &["coulomb"], [0, 0, 1, 1, 0, 0, 0, 0], 1.0, true, "電気"),
        u("V", &["volt"], [2, 1, -3, -1, 0, 0, 0, 0], 1.0, true, "電気"),
        u("ohm", &["Ω", "Ohm"], [2, 1, -3, -2, 0, 0, 0, 0], 1.0, true, "電気"),
        u("S", &["siemens"], [-2, -1, 3, 2, 0, 0, 0, 0], 1.0, true, "電気"),
        u("F", &["farad"], [-2, -1, 4, 2, 0, 0, 0, 0], 1.0, true, "電気"),
        u("Wb", &["weber"], [2, 1, -2, -1, 0, 0, 0, 0], 1.0, true, "磁気"),
        u("T", &["tesla"], [0, 1, -2, -1, 0, 0, 0, 0], 1.0, true, "磁気"),
        u("H", &["henry"], [2, 1, -2, -2, 0, 0, 0, 0], 1.0, true, "磁気"),
        // 放射線
        u("Bq", &["becquerel"], [0, 0, -1, 0, 0, 0, 0, 0], 1.0, true, "放射線"),
        u("Gy", &["gray"], [2, 0, -2, 0, 0, 0, 0, 0], 1.0, true, "放射線"),
        u("Sv", &["sievert"], [2, 0, -2, 0, 0, 0, 0, 0], 1.0, true, "放射線"),
        // 情報
        u("bit", &["b"], INFO, 1.0, true, "情報"),
        u("B", &["byte", "octet"], INFO, 8.0, true, "情報"),
    ]
}

fn find_unit(token: &str) -> Option<UnitDef> {
    for unit in units() {
        if unit.name == token || unit.aliases.contains(&token) {
            return Some(unit);
        }
    }
    None
}

enum Resolved {
    Linear {
        dims: Dims,
        /// 1 この表記 = factor SI
        factor: f64,
        /// 接頭辞を外したときの基本単位の倍率
        base_factor: f64,
        /// 基本単位の記号（m, g, Hz, B …）
        base: &'static str,
        prefixable: bool,
        /// 付いている接頭辞の記号
        prefix_sym: &'static str,
    },
    Temp {
        scale: f64,
        offset: f64,
        name: &'static str,
    },
}

fn resolve(token: &str) -> Result<Resolved, String> {
    let t = token.trim();
    match t {
        "°C" | "℃" | "degC" | "celsius" | "Celsius" => {
            return Ok(Resolved::Temp { scale: 1.0, offset: 273.15, name: "°C" });
        }
        "°F" | "℉" | "degF" | "fahrenheit" | "Fahrenheit" => {
            return Ok(Resolved::Temp { scale: 5.0 / 9.0, offset: 273.15 - 32.0 * 5.0 / 9.0, name: "°F" });
        }
        "K" | "kelvin" | "Kelvin" => {
            return Ok(Resolved::Temp { scale: 1.0, offset: 0.0, name: "K" });
        }
        _ => {}
    }

    // 丸ごと単位名として一致するか（m → メートル、min → 分、cd → カンデラ）
    if let Some(unit) = find_unit(t) {
        return Ok(Resolved::Linear {
            dims: unit.dims,
            factor: unit.factor,
            base_factor: unit.factor,
            base: unit.name,
            prefixable: unit.prefixable,
            prefix_sym: "",
        });
    }

    // 「接頭辞 + 単位」に分ける（km, GHz, µm, ms, kB …）
    for (sym, mult) in PREFIXES {
        let rest = match t.strip_prefix(sym) {
            Some(rest) if !rest.is_empty() => rest,
            _ => continue,
        };
        if let Some(unit) = find_unit(rest) {
            return Ok(Resolved::Linear {
                dims: unit.dims,
                factor: mult * unit.factor,
                base_factor: unit.factor,
                base: unit.name,
                prefixable: unit.prefixable,
                prefix_sym: sym,
            });
        }
    }

    Err(format!("知らない単位です: {t}"))
}

fn parse_value(raw: &str) -> Result<f64, String> {
    let raw = raw.trim();
    // 「1/2」のような分数も許す
    if let Some((a, b)) = raw.split_once('/') {
        let a: f64 = a.trim().parse().map_err(|_| format!("値を数として読めません: {raw}"))?;
        let b: f64 = b.trim().parse().map_err(|_| format!("値を数として読めません: {raw}"))?;
        return Ok(a / b);
    }
    raw.parse().map_err(|_| format!("値を数として読めません: {raw}"))
}

fn convert(value_raw: &str, from: &str, to: &str) -> Result<String, String> {
    let value = parse_value(value_raw)?;
    let source = resolve(from)?;
    let target = resolve(to)?;

    match (source, target) {
        (
            Resolved::Temp { scale: fs, offset: fo, .. },
            Resolved::Temp { scale: ts, offset: to_off, name: tn },
        ) => {
            let kelvin = value * fs + fo;
            let out = (kelvin - to_off) / ts;
            Ok(format!("{} {tn}", fmt_num(out)))
        }
        (
            Resolved::Linear { dims: fd, factor: ff, .. },
            Resolved::Linear {
                dims: td,
                factor: tf,
                base_factor: tbf,
                base: tb,
                prefixable,
                prefix_sym: tpsym,
            },
        ) => {
            if fd != td {
                return Err(format!("次元が違います（{from} と {to}）"));
            }
            let out = value * ff / tf;
            let mut line = format!("{} {}", fmt_num(out), to);
            // 読みやすい接頭辞でも示す（例: 3.5 µm）
            if prefixable {
                let base_val = value * ff / tbf;
                if let Some((sym, shown)) = best_prefix(base_val) {
                    if sym != tpsym || tpsym == "" {
                        line.push_str(&format!("\n≈ {} {sym}{tb}", fmt_num(shown)));
                    }
                }
            }
            Ok(line)
        }
        _ => Err(format!("温度と他の単位は混ぜられません（{from} と {to}）")),
    }
}

/// 10^n の接頭辞のうち、値が [1, 1000) に収まるものを選ぶ。
fn best_prefix(x: f64) -> Option<(&'static str, f64)> {
    if x == 0.0 || !x.is_finite() {
        return None;
    }
    const STEPS: &[(&str, i32)] = &[
        ("Y", 24), ("Z", 21), ("E", 18), ("P", 15), ("T", 12), ("G", 9), ("M", 6),
        ("k", 3), ("", 0), ("m", -3), ("µ", -6), ("n", -9), ("p", -12), ("f", -15),
        ("a", -18), ("z", -21), ("y", -24),
    ];
    let exp = x.abs().log10().floor() as i32;
    let mut chosen = ("y", -24);
    for &(sym, step) in STEPS {
        if exp >= step {
            chosen = (sym, step);
            break;
        }
    }
    let scale = 10f64.powi(chosen.1);
    Some((chosen.0, x / scale))
}

fn fmt_num(x: f64) -> String {
    if x == 0.0 {
        return "0".to_string();
    }
    if !x.is_finite() {
        return format!("{x}");
    }
    let a = x.abs();
    if (1e-4..1e7).contains(&a) {
        let s = format!("{x:.10}");
        if s.contains('.') {
            return s.trim_end_matches('0').trim_end_matches('.').to_string();
        }
        s
    } else {
        let s = format!("{x:.6e}");
        match s.find('e') {
            Some(pos) => {
                let (mantissa, exponent) = s.split_at(pos);
                let mantissa = mantissa.trim_end_matches('0').trim_end_matches('.');
                format!("{mantissa}{exponent}")
            }
            None => s,
        }
    }
}

fn prefix_value(token: &str) -> Option<f64> {
    let t = token.trim();
    for (sym, mult) in PREFIXES {
        if *sym == t {
            return Some(*mult);
        }
    }
    for (name, mult) in PREFIX_NAMES {
        if *name == t {
            return Some(*mult);
        }
    }
    None
}

fn scale_cmd(value_raw: &str, from: &str, to: &str) -> Result<String, String> {
    let value = parse_value(value_raw)?;
    let a = prefix_value(from).ok_or_else(|| format!("知らない接頭辞です: {from}"))?;
    let b = prefix_value(to).ok_or_else(|| format!("知らない接頭辞です: {to}"))?;
    Ok(format!("{}", fmt_num(value * a / b)))
}

fn base_cmd(value_raw: &str, base_raw: &str) -> Result<String, String> {
    let base: u32 = base_raw
        .trim()
        .parse()
        .map_err(|_| format!("基数を数として読めません: {base_raw}"))?;
    let value = i128::from_str_radix(value_raw.trim().trim_start_matches("0x"), base)
        .map_err(|_| format!("{value_raw} を {base} 進数として読めません"))?;
    let sign = if value < 0 { "-" } else { "" };
    let magnitude = value.unsigned_abs();
    Ok(format!(
        "{sign}{magnitude:b}(2)  {sign}{magnitude:o}(8)  {sign}{magnitude}(10)  {sign}{magnitude:X}(16)"
    ))
}

fn print_help() {
    println!(
        "uconv — SI接頭辞と単位を爆速で変換する\n\
\n\
  使い方:\n\
    uconv <値> <変換前> <変換後>    単位変換（接頭辞つき）\n\
    uconv scale <値> <接頭辞A> <接頭辞B>  接頭辞スケーリング\n\
    uconv base <値> <基数>          2/8/10/16 進数へ\n\
    uconv --prefixes               接頭辞の一覧\n\
    uconv --units                  単位の一覧\n\
    uconv                          対話モード\n\
\n\
  例:\n\
    uconv 5 km mile           → 3.1068559612 mi\n\
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
    println!("接頭辞（10^n。Ki〜Ei は 1024^n の2進接頭辞）\n");
    println!("{:<10} {:<4} {}", "名前", "記号", "倍率");
    let rows: &[(&str, &str, f64)] = &[
        ("yotta", "Y", 1e24), ("zetta", "Z", 1e21), ("exa", "E", 1e18),
        ("peta", "P", 1e15), ("tera", "T", 1e12), ("giga", "G", 1e9),
        ("mega", "M", 1e6), ("kilo", "k", 1e3), ("hecto", "h", 1e2),
        ("deca", "da", 1e1), ("deci", "d", 1e-1), ("centi", "c", 1e-2),
        ("milli", "m", 1e-3), ("micro", "µ/u", 1e-6), ("nano", "n", 1e-9),
        ("pico", "p", 1e-12), ("femto", "f", 1e-15), ("atto", "a", 1e-18),
        ("zepto", "z", 1e-21), ("yocto", "y", 1e-24),
        ("kibi", "Ki", 1024.0), ("mebi", "Mi", 1_048_576.0),
        ("gibi", "Gi", 1_073_741_824.0), ("tebi", "Ti", 1_099_511_627_776.0),
    ];
    for (name, sym, mult) in rows {
        println!("{name:<10} {sym:<4} {mult:e}");
    }
}

fn print_units() {
    let all = units();
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
            ["scale", v, a, b] => scale_cmd(v, a, b),
            ["base", v, base] => base_cmd(v, base),
            [v, from, to] => convert(v, from, to),
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
            [_, v, a, b] => scale_cmd(v, a, b).map(|text| println!("{text}")),
            _ => Err("使い方: uconv scale <値> <接頭辞A> <接頭辞B>".to_string()),
        },
        "base" => match args.as_slice() {
            [_, v, base] => base_cmd(v, base).map(|text| println!("{text}")),
            _ => Err("使い方: uconv base <値> <基数>".to_string()),
        },
        _ => match args.as_slice() {
            [v, from, to] => convert(v, from, to).map(|text| println!("{text}")),
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

    fn first_token(text: &str) -> f64 {
        text.split_whitespace().next().unwrap().parse().unwrap()
    }

    #[test]
    fn length_conversion() {
        let out = convert("5", "km", "mile").unwrap();
        assert!((first_token(&out) - 3.1068559611866697).abs() < 1e-9);
    }

    #[test]
    fn prefix_scaling() {
        assert_eq!(scale_cmd("5", "m", "p").unwrap(), "5e9");
        assert_eq!(scale_cmd("3", "k", "M").unwrap(), "0.003");
    }

    #[test]
    fn dimension_mismatch_is_rejected() {
        assert!(convert("1", "km", "lb").is_err());
    }

    #[test]
    fn temperature_is_affine() {
        assert_eq!(convert("100", "degC", "degF").unwrap().trim(), "212 °F");
        assert_eq!(convert("98.6", "degF", "K").unwrap().trim(), "310.15 K");
    }

    #[test]
    fn binary_prefixes() {
        assert_eq!(convert("1", "GiB", "MiB").unwrap().lines().next().unwrap(), "1024 MiB");
    }

    #[test]
    fn base_conversion() {
        assert!(base_cmd("ff", "16").unwrap().contains("255(10)"));
    }
}

