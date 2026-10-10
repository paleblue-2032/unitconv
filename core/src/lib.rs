//! unitconv のコア。単位・SI接頭辞の変換ロジックだけをここに置く。
//!
//! このクレートは 2 通りの使われ方をする:
//! * `uconv` CLI が `lib` として使う
//! * C# / 他言語の GUI が `cdylib`（`libunitconv_core.so`）として P/Invoke で呼ぶ
//!
//! 外部クレートへの依存はゼロ。JSON も自前で組み立てる。

pub mod ffi;

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

pub struct UnitDef {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub dims: Dims,
    /// 1 <単位> = factor [SI基本単位]
    pub factor: f64,
    /// SI接頭辞を付けられるか（m, g, s, Hz, B …）
    pub prefixable: bool,
    pub group: &'static str,
}

/// 接頭辞（記号）。2文字を先に試すため、その順で並べる。
pub const PREFIXES: &[(&str, f64)] = &[
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
pub const PREFIX_NAMES: &[(&str, f64)] = &[
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

pub fn units() -> Vec<UnitDef> {
    vec![
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
        u("g", &["gram", "gramme"], MASS, 1e-3, true, "質量"),
        u("t", &["tonne", "ton"], MASS, 1e3, false, "質量"),
        u("lb", &["pound"], MASS, 0.453_592_37, false, "質量"),
        u("oz", &["ounce"], MASS, 0.028_349_523_125, false, "質量"),
        u("u", &["amu", "dalton", "Da"], MASS, 1.660_539_066_60e-27, false, "質量"),
        u("s", &["sec", "second"], TIME, 1.0, true, "時間"),
        u("min", &[], TIME, 60.0, false, "時間"),
        u("h", &["hr", "hour"], TIME, 3600.0, false, "時間"),
        u("day", &["d"], TIME, 86400.0, false, "時間"),
        u("wk", &["week"], TIME, 604_800.0, false, "時間"),
        u("yr", &["year"], TIME, 31_557_600.0, false, "時間"),
        u("A", &["ampere"], CURR, 1.0, true, "電気"),
        u("mol", &["mole"], AMOUNT, 1.0, true, "物量"),
        u("cd", &["candela"], LUM, 1.0, true, "光度"),
        u("Hz", &["hertz"], [0, 0, -1, 0, 0, 0, 0, 0], 1.0, true, "振動数"),
        u("ha", &["hectare"], [2, 0, 0, 0, 0, 0, 0, 0], 1e4, false, "面積"),
        u("acre", &[], [2, 0, 0, 0, 0, 0, 0, 0], 4046.856_422_4, false, "面積"),
        u("L", &["l", "ltr", "liter", "litre"], [3, 0, 0, 0, 0, 0, 0, 0], 1e-3, true, "体積"),
        u("gal", &["gallon"], [3, 0, 0, 0, 0, 0, 0, 0], 0.003_785_411_784, false, "体積"),
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
        u("kph", &["kmh"], [1, 0, -1, 0, 0, 0, 0, 0], 1.0 / 3.6, false, "速さ"),
        u("mph", &[], [1, 0, -1, 0, 0, 0, 0, 0], 0.44704, false, "速さ"),
        u("kn", &["knot"], [1, 0, -1, 0, 0, 0, 0, 0], 1852.0 / 3600.0, false, "速さ"),
        u("c", &["lightspeed"], [1, 0, -1, 0, 0, 0, 0, 0], 299_792_458.0, false, "速さ"),
        u("g0", &["gee"], [1, 0, -2, 0, 0, 0, 0, 0], 9.80665, false, "加速度"),
        u("C", &["coulomb"], [0, 0, 1, 1, 0, 0, 0, 0], 1.0, true, "電気"),
        u("V", &["volt"], [2, 1, -3, -1, 0, 0, 0, 0], 1.0, true, "電気"),
        u("ohm", &["Ω", "Ohm"], [2, 1, -3, -2, 0, 0, 0, 0], 1.0, true, "電気"),
        u("S", &["siemens"], [-2, -1, 3, 2, 0, 0, 0, 0], 1.0, true, "電気"),
        u("F", &["farad"], [-2, -1, 4, 2, 0, 0, 0, 0], 1.0, true, "電気"),
        u("Wb", &["weber"], [2, 1, -2, -1, 0, 0, 0, 0], 1.0, true, "磁気"),
        u("T", &["tesla"], [0, 1, -2, -1, 0, 0, 0, 0], 1.0, true, "磁気"),
        u("H", &["henry"], [2, 1, -2, -2, 0, 0, 0, 0], 1.0, true, "磁気"),
        u("Bq", &["becquerel"], [0, 0, -1, 0, 0, 0, 0, 0], 1.0, true, "放射線"),
        u("Gy", &["gray"], [2, 0, -2, 0, 0, 0, 0, 0], 1.0, true, "放射線"),
        u("Sv", &["sievert"], [2, 0, -2, 0, 0, 0, 0, 0], 1.0, true, "放射線"),
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
        factor: f64,
        base_factor: f64,
        base: &'static str,
        prefixable: bool,
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

pub fn parse_value(raw: &str) -> Result<f64, String> {
    let raw = raw.trim();
    if let Some((a, b)) = raw.split_once('/') {
        let a: f64 = a.trim().parse().map_err(|_| format!("値を数として読めません: {raw}"))?;
        let b: f64 = b.trim().parse().map_err(|_| format!("値を数として読めません: {raw}"))?;
        return Ok(a / b);
    }
    raw.parse().map_err(|_| format!("値を数として読めません: {raw}"))
}

/// 変換結果。`ok` が false のときは `error` にメッセージが入る。
#[derive(Clone, Debug, Default)]
pub struct ConvertResult {
    pub ok: bool,
    pub value: Option<f64>,
    pub unit: Option<String>,
    pub pretty: Option<String>,
    pub error: Option<String>,
}

impl ConvertResult {
    fn ok(value: f64, unit: Option<String>, pretty: Option<String>) -> Self {
        Self { ok: true, value: Some(value), unit, pretty, error: None }
    }
    fn err(message: String) -> Self {
        Self { ok: false, value: None, unit: None, pretty: None, error: Some(message) }
    }

    /// FFI で返す JSON。`{"ok":..,"value":..,"unit":..,"pretty":..,"error":..}`
    pub fn to_json(&self) -> String {
        let value = match self.value {
            Some(v) if v.is_finite() => format!("{v}"),
            _ => "null".to_string(),
        };
        format!(
            "{{\"ok\":{},\"value\":{},\"unit\":{},\"pretty\":{},\"error\":{}}}",
            self.ok,
            value,
            opt_json(&self.unit),
            opt_json(&self.pretty),
            opt_json(&self.error)
        )
    }
}

fn opt_json(value: &Option<String>) -> String {
    match value {
        Some(text) => format!("\"{}\"", escape(text)),
        None => "null".to_string(),
    }
}

fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// 単位変換。CLI と FFI の共通入口。
pub fn convert(value_raw: &str, from: &str, to: &str) -> ConvertResult {
    let value = match parse_value(value_raw) {
        Ok(value) => value,
        Err(message) => return ConvertResult::err(message),
    };
    let source = match resolve(from) {
        Ok(resolved) => resolved,
        Err(message) => return ConvertResult::err(message),
    };
    let target = match resolve(to) {
        Ok(resolved) => resolved,
        Err(message) => return ConvertResult::err(message),
    };

    match (source, target) {
        (
            Resolved::Temp { scale: fs, offset: fo, .. },
            Resolved::Temp { scale: ts, offset: to_off, name: tn },
        ) => {
            let kelvin = value * fs + fo;
            ConvertResult::ok((kelvin - to_off) / ts, Some(tn.to_string()), None)
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
                return ConvertResult::err(format!("次元が違います（{from} と {to}）"));
            }
            let out = value * ff / tf;
            let mut pretty = None;
            if prefixable {
                let base_val = value * ff / tbf;
                if let Some((sym, shown)) = best_prefix(base_val) {
                    if sym != tpsym || tpsym.is_empty() {
                        pretty = Some(format!("{} {sym}{tb}", fmt_num(shown)));
                    }
                }
            }
            ConvertResult::ok(out, Some(to.to_string()), pretty)
        }
        _ => ConvertResult::err(format!("温度と他の単位は混ぜられません（{from} と {to}）")),
    }
}

/// 接頭辞だけのスケーリング（ミリ→ピコ など）。
pub fn scale(value_raw: &str, from: &str, to: &str) -> ConvertResult {
    let value = match parse_value(value_raw) {
        Ok(value) => value,
        Err(message) => return ConvertResult::err(message),
    };
    let a = match prefix_value(from) {
        Some(v) => v,
        None => return ConvertResult::err(format!("知らない接頭辞です: {from}")),
    };
    let b = match prefix_value(to) {
        Some(v) => v,
        None => return ConvertResult::err(format!("知らない接頭辞です: {to}")),
    };
    ConvertResult::ok(value * a / b, None, None)
}

/// 進数変換（2 / 8 / 10 / 16 をまとめて）。
pub fn base(value_raw: &str, base_raw: &str) -> Result<String, String> {
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

fn best_prefix(x: f64) -> Option<(&'static str, f64)> {
    if x == 0.0 || !x.is_finite() {
        return None;
    }
    const STEPS: &[(&str, i32)] = &[
        ("Y", 24), ("Z", 21), ("E", 18), ("P", 15), ("T", 12), ("G", 9), ("M", 6),
        ("k", 3), ("", 0), ("m", -3), ("μ", -6), ("n", -9), ("p", -12), ("f", -15),
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
    Some((chosen.0, x / 10f64.powi(chosen.1)))
}

pub fn fmt_num(x: f64) -> String {
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

pub fn prefix_value(token: &str) -> Option<f64> {
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

pub struct PrefixInfo {
    pub name: &'static str,
    pub symbol: &'static str,
    pub mult: f64,
}

/// 一覧表示用（名前, 記号, 倍率）。記号は SI の定義どおり。
/// micro は ASCII の "u" ではなくギリシャ文字 μ で見せる。
const PREFIX_ROWS: &[(&str, &str, f64)] = &[
    ("yotta", "Y", 1e24),
    ("zetta", "Z", 1e21),
    ("exa", "E", 1e18),
    ("peta", "P", 1e15),
    ("tera", "T", 1e12),
    ("giga", "G", 1e9),
    ("mega", "M", 1e6),
    ("kilo", "k", 1e3),
    ("hecto", "h", 1e2),
    ("deca", "da", 1e1),
    ("deci", "d", 1e-1),
    ("centi", "c", 1e-2),
    ("milli", "m", 1e-3),
    ("micro", "μ", 1e-6),
    ("nano", "n", 1e-9),
    ("pico", "p", 1e-12),
    ("femto", "f", 1e-15),
    ("atto", "a", 1e-18),
    ("zepto", "z", 1e-21),
    ("yocto", "y", 1e-24),
    ("kibi", "Ki", 1024.0),
    ("mebi", "Mi", 1_048_576.0),
    ("gibi", "Gi", 1_073_741_824.0),
    ("tebi", "Ti", 1_099_511_627_776.0),
];

/// 接頭辞の一覧を大きさ順（大きい順）で返す。
pub fn prefix_table() -> Vec<PrefixInfo> {
    let mut rows: Vec<PrefixInfo> = PREFIX_ROWS
        .iter()
        .map(|(name, symbol, mult)| PrefixInfo { name, symbol, mult: *mult })
        .collect();
    rows.sort_by(|a, b| b.mult.partial_cmp(&a.mult).unwrap_or(std::cmp::Ordering::Equal));
    rows
}

/// 接頭辞一覧の JSON。
pub fn prefixes_json() -> String {
    let rows: Vec<String> = prefix_table()
        .iter()
        .map(|p| {
            format!(
                "{{\"name\":\"{}\",\"symbol\":\"{}\",\"mult\":{}}}",
                escape(p.name),
                escape(p.symbol),
                p.mult
            )
        })
        .collect();
    format!("[{}]", rows.join(","))
}

/// 単位一覧の JSON。
pub fn units_json() -> String {
    let rows: Vec<String> = units()
        .iter()
        .map(|unit| {
            let aliases: Vec<String> =
                unit.aliases.iter().map(|a| format!("\"{}\"", escape(a))).collect();
            format!(
                "{{\"name\":\"{}\",\"group\":\"{}\",\"prefixable\":{},\"aliases\":[{}]}}",
                escape(unit.name),
                escape(unit.group),
                unit.prefixable,
                aliases.join(",")
            )
        })
        .collect();
    format!("[{}]", rows.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn length_conversion() {
        let r = convert("5", "km", "mile");
        assert!(r.ok);
        assert!((r.value.unwrap() - 3.1068559611866697).abs() < 1e-9);
    }

    #[test]
    fn prefix_scaling() {
        assert_eq!(scale("5", "m", "p").value.unwrap(), 5e9);
        assert_eq!(fmt_num(scale("3", "k", "M").value.unwrap()), "0.003");
    }

    #[test]
    fn dimension_mismatch_is_rejected() {
        assert!(!convert("1", "km", "lb").ok);
    }

    #[test]
    fn temperature_is_affine() {
        let r = convert("100", "degC", "degF");
        assert_eq!(fmt_num(r.value.unwrap()), "212");
        let k = convert("98.6", "degF", "K");
        assert_eq!(fmt_num(k.value.unwrap()), "310.15");
    }

    #[test]
    fn binary_prefixes() {
        let r = convert("1", "GiB", "MiB");
        assert_eq!(fmt_num(r.value.unwrap()), "1024");
    }

    #[test]
    fn json_is_wellformed() {
        let json = convert("5", "km", "mile").to_json();
        assert!(json.starts_with("{\"ok\":true"));
        assert!(json.contains("\"unit\":\"mile\""));
    }
}
