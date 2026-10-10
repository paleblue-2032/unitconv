using System.Runtime.InteropServices;

namespace Unitconv;

/// <summary>
/// Rust コア（libunitconv_core.so）への薄い P/Invoke ラッパー。
/// 変換の計算も整形も全部 Rust 側でやる。C# は文字列を渡して JSON を受け取るだけ。
/// </summary>
public static class Native
{
    private const string Lib = "unitconv_core";

    static Native()
    {
        // どの .so を読むかを自分で決める（環境変数 → 出力フォルダ → 既定の探索）。
        NativeLibrary.SetDllImportResolver(typeof(Native).Assembly, (name, _, _) =>
        {
            if (name != Lib)
            {
                return IntPtr.Zero;
            }
            var overridden = Environment.GetEnvironmentVariable("UNITCONV_CORE_LIB");
            if (!string.IsNullOrEmpty(overridden) && File.Exists(overridden))
            {
                return NativeLibrary.Load(overridden);
            }
            var local = Path.Combine(AppContext.BaseDirectory, "libunitconv_core.so");
            if (File.Exists(local))
            {
                return NativeLibrary.Load(local);
            }
            return IntPtr.Zero;
        });
    }

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr unitconv_convert(
        [MarshalAs(UnmanagedType.LPUTF8Str)] string value,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string from,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string to);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr unitconv_scale(
        [MarshalAs(UnmanagedType.LPUTF8Str)] string value,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string from,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string to);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr unitconv_prefixes();

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr unitconv_units();

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    private static extern void unitconv_free(IntPtr pointer);

    private static string Take(IntPtr pointer)
    {
        if (pointer == IntPtr.Zero)
        {
            return "{}";
        }
        try
        {
            return Marshal.PtrToStringUTF8(pointer) ?? "{}";
        }
        finally
        {
            unitconv_free(pointer);
        }
    }

    public static string Convert(string value, string from, string to) =>
        Take(unitconv_convert(value, from, to));

    public static string Scale(string value, string from, string to) =>
        Take(unitconv_scale(value, from, to));

    public static string Prefixes() => Take(unitconv_prefixes());

    public static string Units() => Take(unitconv_units());
}
