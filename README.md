# unitconv

単位と SI 接頭辞（`m` `μ` `p` `k` `M` …）をパッと変換するツールです。
変換ロジックは **Rust コア 1 つ**にまとめてあり、**CLI** と **2 種類の Web UI**
（サーバ型 / クライアント完結）から同じコアを使います。

**デモ:** <https://kanon-server.com/others/unitconv/> （Blazor WebAssembly 版・ブラウザだけで動作）

```sh
uconv 5 km mile         # 3.1068559612 mile
uconv 3.2 GHz MHz       # 3200 MHz   （≈ 3.2 GHz）
uconv 0.0000035 m mm    # 0.0035 mm  （≈ 3.5 μm）
uconv 100 degC degF     # 212 °F
```

## 特徴

- **SI 接頭辞をそのまま扱える** — `km` `GHz` `μm` `ms` `kB` `GiB` のように書けます。
- **次元を検査** — `km → lb` のような次元の違う変換はエラーにします。
- **温度は別扱い** — `degC` / `degF` / `K` をアフィン変換として正しく換算します。
- **読みやすい接頭辞で併記** — `0.0000035 m` を `mm` にすると `≈ 3.5 μm` も表示します。
- **依存クレートゼロ** — Rust コアは外部クレートを使いません（起動は一瞬）。

## 構成

| パス | 中身 |
| --- | --- |
| `core/` | Rust の変換コア（`lib` + `cdylib`）。**変換のすべてはここ**。 |
| `cli/` | Rust 製 CLI `uconv`。 |
| `web/` | ASP.NET Core の Web UI（サーバ型）。コアを P/Invoke で呼ぶ薄いフロント。 |
| `blazor/` | Blazor WebAssembly の Web UI（**クライアント完結**）。コアを wasm で呼ぶ。静的ホスティング向け。 |

```
core/ ──(lib)────────────────▶ cli/    (uconv)
    ├─(cdylib: libunitconv_core.so)─▶ web/    (ASP.NET, P/Invoke)
    └─(wasm: unitconv_core.wasm)────▶ blazor/ (Blazor WASM, JS interop)
```

## 必要なもの

- **Rust ツールチェーン**（`cargo` / `rustc`）— コア・CLI に必須
- **.NET SDK 8** — `web/` と `blazor/` を使う場合
- **`wasm32-unknown-unknown` ターゲット と `wasm-ld`** — Blazor 版の wasm をビルドする場合

```sh
# Rust の wasm ターゲットを追加
rustup target add wasm32-unknown-unknown
# wasm-ld は LLVM の lld に同梱（例: apt install lld / brew install llvm）
```

## インストール

### CLI（`uconv`）

```sh
git clone <このリポジトリ>
cd unitconv

cargo install --path cli     # ~/.cargo/bin/uconv に入ります
uconv 5 km mile
```

ビルドだけして直接使う場合:

```sh
cargo build --release        # バイナリは target/release/uconv
```

### Web（ASP.NET Core・サーバ型）

```sh
cd web
dotnet run                   # → http://127.0.0.1:5080
```

`dotnet build` 時に Rust コアを `cargo build --release` でビルドし、
`libunitconv_core.so` を出力へコピーします（P/Invoke 用）。

### Web（Blazor WebAssembly・クライアント完結）

サーバ不要。初回だけ Rust コアを wasm にビルドします。

```sh
# 1) Rust コアを wasm に（生成物。リポジトリには含めていません）
cargo build -p unitconv-core --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/unitconv_core.wasm blazor/wwwroot/

# 2) 静的ファイルを publish
cd blazor
dotnet publish -c Release -o publish
# publish/wwwroot をそのまま静的ホスティングに置けば動きます
```

### テスト

```sh
cargo test                   # リポジトリのルートで
```

## 使い方（CLI）

```sh
uconv <値> <変換前> <変換後>      # 単位変換（接頭辞つき）
uconv scale <値> <A> <B>         # 接頭辞スケーリング（ミリ → ピコ など）
uconv base <値> <基数>           # 2 / 8 / 10 / 16 進数へ
uconv --prefixes                # 接頭辞の一覧
uconv --units                   # 単位の一覧
uconv                           # 引数なしで対話モード
```

例:

```sh
uconv 5 km mile         # 3.1068559612 mile
uconv 3.2 GHz MHz       # 3200 MHz  （≈ 3.2 GHz）
uconv 0.0000035 m mm    # 0.0035 mm （≈ 3.5 μm）
uconv 100 degC degF     # 212 °F
uconv 1 GiB MiB         # 1024 MiB
uconv 65 mph kph        # 104.60736 kph
uconv 1 eV J            # 1.602176634e-19 J
uconv scale 5 m p       # 5e9
uconv base ff 16        # 11111111(2) 377(8) 255(10) FF(16)
```

## Web API（`web/` のサーバ型）

| エンドポイント | 内容 |
| --- | --- |
| `GET /api/convert?value=5&from=km&to=mile` | 単位変換 |
| `GET /api/scale?value=5&from=m&to=p` | 接頭辞スケーリング |
| `GET /api/prefixes` | 接頭辞の一覧（JSON 配列） |
| `GET /api/units` | 単位の一覧（JSON 配列） |

`/api/convert` と `/api/scale` は次の JSON を返します:

```json
{ "ok": true, "value": 3.1068559612, "unit": "mile", "pretty": null, "error": null }
```

## 対応している単位（抜粋）

長さ `m mi ft in yd nmi Å au ly pc` / 質量 `g t lb oz u(amu)` /
時間 `s min h day wk yr` / 電気 `A C V ohm S F Wb T H` /
エネルギー `J cal kcal Wh eV` / 仕事率 `W hp` / 圧力 `Pa bar atm mmHg psi` /
速さ `kph mph kn c` / 振動数 `Hz` / 面積 `ha acre` / 体積 `L gal` /
情報 `bit B`（2進接頭辞 `KiB` 等も） / 放射線 `Bq Gy Sv` …ほか。

`uconv --units` で全一覧を表示します。

## 温度の注意

`C` はクーロン、`F` はファラド、`K` はケルビンです。温度として使うときは
`degC` / `degF` / `K`（`°C` / `°F` も可）と書いてください。

| 書き方 | 意味 |
| --- | --- |
| `degC` / `°C` / `celsius` | 摂氏 |
| `degF` / `°F` / `fahrenheit` | 華氏 |
| `K` / `kelvin` | 絶対温度 |

## 仕組み

単位トークンを解決して換算します:

1. トークンを「単位そのもの」として照合（`m`→メートル、`min`→分）
2. 一致しなければ「接頭辞＋単位」に分解（`GHz`→ G + Hz、`μm`→ μ + m）
3. 次元ベクトル（m, kg, s, A, K, mol, cd, bit）が一致するか確認して比率で換算
4. 温度だけは `scale` / `offset` を持つ別表現にし、K を経由して変換

Rust コアは C ABI として公開しています（`unitconv_convert` / `unitconv_scale` /
`unitconv_prefixes` / `unitconv_units`。いずれも UTF-8 の JSON 文字列を返し、
呼び出し側が `unitconv_free` で解放）。CLI は `lib` として、ASP.NET 版は P/Invoke で、
Blazor 版は JavaScript interop 経由で wasm として、それぞれ同じコアを呼びます。

## メモ

- wasm / `bin` / `obj` / `publish` などのビルド生成物は `.gitignore` 済みです
  （`blazor/wwwroot/unitconv_core.wasm` は生成物なのでコミットしていません。上の手順でビルドしてください）。
- Blazor 版をサブパスに置く場合は `blazor/wwwroot/index.html` の `<base href="...">` を合わせてください。

## ライセンス

MIT（[LICENSE](LICENSE)）
