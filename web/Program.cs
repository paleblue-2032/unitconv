using Unitconv;

var builder = WebApplication.CreateBuilder(args);
var app = builder.Build();

app.UseDefaultFiles();
app.UseStaticFiles();

app.MapGet("/api/convert", (string? value, string? from, string? to) =>
{
    if (string.IsNullOrWhiteSpace(value) || string.IsNullOrWhiteSpace(from) || string.IsNullOrWhiteSpace(to))
        return Results.Json(new ConvertResult(false, null, null, null, "value, from, to を指定してください"));
    return Results.Json(Converter.Convert(value, from, to));
});

app.MapGet("/api/scale", (string? value, string? from, string? to) =>
{
    if (string.IsNullOrWhiteSpace(value) || string.IsNullOrWhiteSpace(from) || string.IsNullOrWhiteSpace(to))
        return Results.Json(new ConvertResult(false, null, null, null, "value, from, to を指定してください"));
    return Results.Json(Converter.Scale(value, from, to));
});

app.MapGet("/api/prefixes", () => Results.Json(Converter.Prefixes()));
app.MapGet("/api/units", () => Results.Json(Converter.UnitList()));

app.Run("http://127.0.0.1:5080");
