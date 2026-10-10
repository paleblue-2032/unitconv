using System.Globalization;

namespace Unitconv;

/// <summary>変換結果（API の戻り値）。</summary>
public sealed record ConvertResult(bool Ok, double? Value, string? Unit, string? Pretty, string? Error);

public sealed record PrefixInfo(string Name, string Symbol, double Mult);

public sealed record UnitInfo(string Name, string Group, bool Prefixable, string[] Aliases);

/// <summary>変換できないときの例外。</summary>
public sealed class ConvException(string message) : Exception(message);

/// <summary>解決済みの単位（線形 or 温度）。</summary>
public sealed class Resolved
{
    public bool IsTemp;
    public int[] Dims = new int[8];
    public double Factor;
    public double BaseFactor;
    public string Base = "";
    public bool Prefixable;
    public string PrefixSym = "";
    public double TempScale;
    public double TempOffset;
    public string TempName = "";
}

public sealed record UnitDef(string Name, string[] Aliases, int[] Dims, double Factor, bool Prefixable, string Group);

public static class Converter
{
    // 次元: [m, kg, s, A, K, mol, cd, bit]
    static readonly int[] Len = { 1, 0, 0, 0, 0, 0, 0, 0 };
    static readonly int[] Mass = { 0, 1, 0, 0, 0, 0, 0, 0 };
    static readonly int[] Time = { 0, 0, 1, 0, 0, 0, 0, 0 };
    static readonly int[] Curr = { 0, 0, 0, 1, 0, 0, 0, 0 };
    static readonly int[] Amount = { 0, 0, 0, 0, 0, 1, 0, 0 };
    static readonly int[] Lum = { 0, 0, 0, 0, 0, 0, 1, 0 };
    static readonly int[] Info = { 0, 0, 0, 0, 0, 0, 0, 1 };

    static readonly UnitDef[] Units =
    {
        new("m", new[] { "meter", "metre" }, Len, 1.0, true, "長さ"),
        new("mi", new[] { "mile" }, Len, 1609.344, false, "長さ"),
        new("ft", new[] { "foot", "feet" }, Len, 0.3048, false, "長さ"),
        new("in", new[] { "inch" }, Len, 0.0254, false, "長さ"),
        new("yd", new[] { "yard" }, Len, 0.9144, false, "長さ"),
        new("nmi", new[] { "nauticalmile" }, Len, 1852.0, false, "長さ"),
        new("Å", new[] { "angstrom" }, Len, 1e-10, false, "長さ"),
        new("au", new[] { "astronomicalunit" }, Len, 1.495978707e11, false, "長さ"),
        new("ly", new[] { "lightyear" }, Len, 9.4607304725808e15, false, "長さ"),
        new("pc", new[] { "parsec" }, Len, 3.0856775814913673e16, false, "長さ"),

        new("g", new[] { "gram", "gramme" }, Mass, 1e-3, true, "質量"),
        new("t", new[] { "tonne", "ton" }, Mass, 1e3, false, "質量"),
        new("lb", new[] { "pound" }, Mass, 0.45359237, false, "質量"),
        new("oz", new[] { "ounce" }, Mass, 0.028349523125, false, "質量"),
        new("u", new[] { "amu", "dalton", "Da" }, Mass, 1.66053906660e-27, false, "質量"),

        new("s", new[] { "sec", "second" }, Time, 1.0, true, "時間"),
        new("min", Array.Empty<string>(), Time, 60.0, false, "時間"),
        new("h", new[] { "hr", "hour" }, Time, 3600.0, false, "時間"),
        new("day", new[] { "d" }, Time, 86400.0, false, "時間"),
        new("wk", new[] { "week" }, Time, 604800.0, false, "時間"),
        new("yr", new[] { "year" }, Time, 31557600.0, false, "時間"),

        new("A", new[] { "ampere" }, Curr, 1.0, true, "電気"),
        new("mol", new[] { "mole" }, Amount, 1.0, true, "物量"),
        new("cd", new[] { "candela" }, Lum, 1.0, true, "光度"),

        new("Hz", new[] { "hertz" }, new[] { 0, 0, -1, 0, 0, 0, 0, 0 }, 1.0, true, "振動数"),

        new("ha", new[] { "hectare" }, new[] { 2, 0, 0, 0, 0, 0, 0, 0 }, 1e4, false, "面積"),
        new("acre", Array.Empty<string>(), new[] { 2, 0, 0, 0, 0, 0, 0, 0 }, 4046.8564224, false, "面積"),

        new("L", new[] { "l", "ltr", "liter", "litre" }, new[] { 3, 0, 0, 0, 0, 0, 0, 0 }, 1e-3, true, "体積"),
        new("gal", new[] { "gallon" }, new[] { 3, 0, 0, 0, 0, 0, 0, 0 }, 0.003785411784, false, "体積"),

        new("N", new[] { "newton" }, new[] { 1, 1, -2, 0, 0, 0, 0, 0 }, 1.0, true, "力"),
        new("J", new[] { "joule" }, new[] { 2, 1, -2, 0, 0, 0, 0, 0 }, 1.0, true, "エネルギー"),
        new("cal", new[] { "calorie" }, new[] { 2, 1, -2, 0, 0, 0, 0, 0 }, 4.184, false, "エネルギー"),
        new("kcal", Array.Empty<string>(), new[] { 2, 1, -2, 0, 0, 0, 0, 0 }, 4184.0, false, "エネルギー"),
        new("Wh", Array.Empty<string>(), new[] { 2, 1, -2, 0, 0, 0, 0, 0 }, 3600.0, true, "エネルギー"),
        new("eV", new[] { "electronvolt" }, new[] { 2, 1, -2, 0, 0, 0, 0, 0 }, 1.602176634e-19, true, "エネルギー"),
        new("W", new[] { "watt" }, new[] { 2, 1, -3, 0, 0, 0, 0, 0 }, 1.0, true, "仕事率"),
        new("hp", new[] { "horsepower" }, new[] { 2, 1, -3, 0, 0, 0, 0, 0 }, 745.6998715822702, false, "仕事率"),
        new("Pa", new[] { "pascal" }, new[] { -1, 1, -2, 0, 0, 0, 0, 0 }, 1.0, true, "圧力"),
        new("bar", Array.Empty<string>(), new[] { -1, 1, -2, 0, 0, 0, 0, 0 }, 1e5, false, "圧力"),
        new("atm", new[] { "atmosphere" }, new[] { -1, 1, -2, 0, 0, 0, 0, 0 }, 101325.0, false, "圧力"),
        new("mmHg", Array.Empty<string>(), new[] { -1, 1, -2, 0, 0, 0, 0, 0 }, 133.322387415, false, "圧力"),
        new("psi", Array.Empty<string>(), new[] { -1, 1, -2, 0, 0, 0, 0, 0 }, 6894.757293168, false, "圧力"),

        new("kph", new[] { "kmh" }, new[] { 1, 0, -1, 0, 0, 0, 0, 0 }, 1.0 / 3.6, false, "速さ"),
        new("mph", Array.Empty<string>(), new[] { 1, 0, -1, 0, 0, 0, 0, 0 }, 0.44704, false, "速さ"),
        new("kn", new[] { "knot" }, new[] { 1, 0, -1, 0, 0, 0, 0, 0 }, 1852.0 / 3600.0, false, "速さ"),
        new("c", new[] { "lightspeed" }, new[] { 1, 0, -1, 0, 0, 0, 0, 0 }, 299792458.0, false, "速さ"),
        new("g0", new[] { "gee" }, new[] { 1, 0, -2, 0, 0, 0, 0, 0 }, 9.80665, false, "加速度"),

        new("C", new[] { "coulomb" }, new[] { 0, 0, 1, 1, 0, 0, 0, 0 }, 1.0, true, "電気"),
        new("V", new[] { "volt" }, new[] { 2, 1, -3, -1, 0, 0, 0, 0 }, 1.0, true, "電気"),
        new("ohm", new[] { "Ω", "Ohm" }, new[] { 2, 1, -3, -2, 0, 0, 0, 0 }, 1.0, true, "電気"),
        new("S", new[] { "siemens" }, new[] { -2, -1, 3, 2, 0, 0, 0, 0 }, 1.0, true, "電気"),
        new("F", new[] { "farad" }, new[] { -2, -1, 4, 2, 0, 0, 0, 0 }, 1.0, true, "電気"),
        new("Wb", new[] { "weber" }, new[] { 2, 1, -2, -1, 0, 0, 0, 0 }, 1.0, true, "磁気"),
        new("T", new[] { "tesla" }, new[] { 0, 1, -2, -1, 0, 0, 0, 0 }, 1.0, true, "磁気"),
        new("H", new[] { "henry" }, new[] { 2, 1, -2, -2, 0, 0, 0, 0 }, 1.0, true, "磁気"),

        new("Bq", new[] { "becquerel" }, new[] { 0, 0, -1, 0, 0, 0, 0, 0 }, 1.0, true, "放射線"),
        new("Gy", new[] { "gray" }, new[] { 2, 0, -2, 0, 0, 0, 0, 0 }, 1.0, true, "放射線"),
        new("Sv", new[] { "sievert" }, new[] { 2, 0, -2, 0, 0, 0, 0, 0 }, 1.0, true, "放射線"),

        new("bit", new[] { "b" }, Info, 1.0, true, "情報"),
        new("B", new[] { "byte", "octet" }, Info, 8.0, true, "情報"),
    };

    // 接頭辞（記号）。2文字を先に試す。
    static readonly (string Sym, double Mult)[] Prefix =
    {
        ("da", 1e1),
        ("Ki", 1024.0), ("Mi", 1048576.0), ("Gi", 1073741824.0),
        ("Ti", 1099511627776.0), ("Pi", 1125899906842624.0), ("Ei", 1.152921504606847e18),
        ("Y", 1e24), ("Z", 1e21), ("E", 1e18), ("P", 1e15), ("T", 1e12), ("G", 1e9),
        ("M", 1e6), ("k", 1e3), ("K", 1e3), ("h", 1e2),
        ("d", 1e-1), ("c", 1e-2), ("m", 1e-3),
        ("µ", 1e-6), ("μ", 1e-6), ("u", 1e-6),
        ("n", 1e-9), ("p", 1e-12), ("f", 1e-15), ("a", 1e-18), ("z", 1e-21), ("y", 1e-24),
    };

    static readonly (string Name, string Sym, double Mult)[] PrefixTable =
    {
        ("yotta", "Y", 1e24), ("zetta", "Z", 1e21), ("exa", "E", 1e18), ("peta", "P", 1e15),
        ("tera", "T", 1e12), ("giga", "G", 1e9), ("mega", "M", 1e6), ("kilo", "k", 1e3),
        ("hecto", "h", 1e2), ("deca", "da", 1e1), ("deci", "d", 1e-1), ("centi", "c", 1e-2),
        ("milli", "m", 1e-3), ("micro", "µ/u", 1e-6), ("nano", "n", 1e-9), ("pico", "p", 1e-12),
        ("femto", "f", 1e-15), ("atto", "a", 1e-18), ("zepto", "z", 1e-21), ("yocto", "y", 1e-24),
        ("kibi", "Ki", 1024.0), ("mebi", "Mi", 1048576.0), ("gibi", "Gi", 1073741824.0),
        ("tebi", "Ti", 1099511627776.0),
    };

    static UnitDef? FindUnit(string token)
    {
        foreach (var unit in Units)
        {
            if (unit.Name == token) return unit;
            foreach (var alias in unit.Aliases)
                if (alias == token) return unit;
        }
        return null;
    }

    static Resolved Resolve(string token)
    {
        var t = token.Trim();
        switch (t)
        {
            case "°C" or "℃" or "degC" or "celsius" or "Celsius":
                return new Resolved { IsTemp = true, TempScale = 1.0, TempOffset = 273.15, TempName = "°C" };
            case "°F" or "℉" or "degF" or "fahrenheit" or "Fahrenheit":
                return new Resolved { IsTemp = true, TempScale = 5.0 / 9.0, TempOffset = 273.15 - 32.0 * 5.0 / 9.0, TempName = "°F" };
            case "K" or "kelvin" or "Kelvin":
                return new Resolved { IsTemp = true, TempScale = 1.0, TempOffset = 0.0, TempName = "K" };
        }

        var whole = FindUnit(t);
        if (whole is not null)
        {
            return new Resolved
            {
                Dims = whole.Dims,
                Factor = whole.Factor,
                BaseFactor = whole.Factor,
                Base = whole.Name,
                Prefixable = whole.Prefixable,
                PrefixSym = "",
            };
        }

        foreach (var (sym, mult) in Prefix)
        {
            if (!t.StartsWith(sym, StringComparison.Ordinal) || t.Length == sym.Length) continue;
            var rest = t[sym.Length..];
            var unit = FindUnit(rest);
            if (unit is not null)
            {
                return new Resolved
                {
                    Dims = unit.Dims,
                    Factor = mult * unit.Factor,
                    BaseFactor = unit.Factor,
                    Base = unit.Name,
                    Prefixable = unit.Prefixable,
                    PrefixSym = sym,
                };
            }
        }

        throw new ConvException($"知らない単位です: {t}");
    }

    static double ParseValue(string raw)
    {
        var r = raw.Trim();
        var slash = r.IndexOf('/');
        if (slash >= 0)
        {
            var a = ParseSingle(r[..slash]);
            var b = ParseSingle(r[(slash + 1)..]);
            return a / b;
        }
        return ParseSingle(r);
    }

    static double ParseSingle(string s)
    {
        if (double.TryParse(s.Trim(), NumberStyles.Float, CultureInfo.InvariantCulture, out var value))
            return value;
        throw new ConvException($"値を数として読めません: {s.Trim()}");
    }

    public static ConvertResult Convert(string valueRaw, string from, string to)
    {
        try
        {
            var value = ParseValue(valueRaw);
            var source = Resolve(from);
            var target = Resolve(to);

            if (source.IsTemp && target.IsTemp)
            {
                var kelvin = value * source.TempScale + source.TempOffset;
                var output = (kelvin - target.TempOffset) / target.TempScale;
                return new ConvertResult(true, output, target.TempName, null, null);
            }
            if (source.IsTemp || target.IsTemp)
            {
                return new ConvertResult(false, null, null, null, $"温度と他の単位は混ぜられません（{from} と {to}）");
            }

            if (!source.Dims.SequenceEqual(target.Dims))
            {
                return new ConvertResult(false, null, null, null, $"次元が違います（{from} と {to}）");
            }

            var result = value * source.Factor / target.Factor;
            string? pretty = null;
            if (target.Prefixable)
            {
                var baseValue = value * source.Factor / target.BaseFactor;
                var (sym, shown) = BestPrefix(baseValue);
                if (sym != target.PrefixSym || target.PrefixSym.Length == 0)
                {
                    pretty = $"{FmtNum(shown)} {sym}{target.Base}";
                }
            }
            return new ConvertResult(true, result, to, pretty, null);
        }
        catch (ConvException error)
        {
            return new ConvertResult(false, null, null, null, error.Message);
        }
    }

    public static ConvertResult Scale(string valueRaw, string from, string to)
    {
        try
        {
            var value = ParseValue(valueRaw);
            var a = PrefixValue(from) ?? throw new ConvException($"知らない接頭辞です: {from}");
            var b = PrefixValue(to) ?? throw new ConvException($"知らない接頭辞です: {to}");
            return new ConvertResult(true, value * a / b, null, null, null);
        }
        catch (ConvException error)
        {
            return new ConvertResult(false, null, null, null, error.Message);
        }
    }

    static double? PrefixValue(string token)
    {
        var t = token.Trim();
        foreach (var (sym, mult) in Prefix)
            if (sym == t) return mult;
        foreach (var (name, sym, mult) in PrefixTable)
            if (name == t || (sym == t && sym != "µ/u")) return mult;
        return null;
    }

    static (string, double) BestPrefix(double x)
    {
        (string, int)[] steps =
        {
            ("Y", 24), ("Z", 21), ("E", 18), ("P", 15), ("T", 12), ("G", 9), ("M", 6),
            ("k", 3), ("", 0), ("m", -3), ("µ", -6), ("n", -9), ("p", -12), ("f", -15),
            ("a", -18), ("z", -21), ("y", -24),
        };
        var exp = (int)Math.Floor(Math.Log10(Math.Abs(x)));
        var chosen = ("y", -24);
        foreach (var step in steps)
        {
            if (exp >= step.Item2)
            {
                chosen = step;
                break;
            }
        }
        return (chosen.Item1, x / Math.Pow(10, chosen.Item2));
    }

    public static string FmtNum(double x)
    {
        if (x == 0) return "0";
        if (double.IsNaN(x) || double.IsInfinity(x)) return x.ToString(CultureInfo.InvariantCulture);
        var a = Math.Abs(x);
        if (a >= 1e-4 && a < 1e7) return x.ToString("0.##########", CultureInfo.InvariantCulture);
        return x.ToString("0.######e+0", CultureInfo.InvariantCulture);
    }

    public static IReadOnlyList<PrefixInfo> Prefixes() =>
        PrefixTable.Select(p => new PrefixInfo(p.Name, p.Sym, p.Mult)).ToList();

    public static IReadOnlyList<UnitInfo> UnitList() =>
        Units.Select(u => new UnitInfo(u.Name, u.Group, u.Prefixable, u.Aliases)).ToList();
}
