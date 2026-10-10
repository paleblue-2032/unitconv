using Microsoft.AspNetCore.StaticFiles;
using Unitconv;

var builder = WebApplication.CreateBuilder(args);
builder.WebHost.UseUrls(Environment.GetEnvironmentVariable("ASPNETCORE_URLS") ?? "http://127.0.0.1:5080");
var app = builder.Build();

// 開発中にブラウザが古い index.html / app.js をキャッシュしないようにする。
var staticOptions = new StaticFileOptions
{
    OnPrepareResponse = context =>
        context.Context.Response.Headers.CacheControl = "no-cache",
};
app.UseDefaultFiles();
app.UseStaticFiles(staticOptions);

// 処理は全部 Rust コア（libunitconv_core.so）。C# はその JSON をそのまま返すだけ。
app.MapGet("/api/convert", (string? value, string? from, string? to) =>
    Results.Text(Native.Convert(value ?? "", from ?? "", to ?? ""), "application/json"));

app.MapGet("/api/scale", (string? value, string? from, string? to) =>
    Results.Text(Native.Scale(value ?? "", from ?? "", to ?? ""), "application/json"));

app.MapGet("/api/prefixes", () => Results.Text(Native.Prefixes(), "application/json"));
app.MapGet("/api/units", () => Results.Text(Native.Units(), "application/json"));

app.Run();
