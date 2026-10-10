# unitconv

単位と SI 接頭辞（m, μ, p, k, M…）をパッと変換するツールです。
高速な Rust コアを、CLI と C#（ASP.NET Core）の Web UI から使えます。

![unitconv の画面](docs/screenshot.png)

## 特徴

- **SI 接頭辞をそのまま扱える** — `km` `GHz` `μm` `ms` `kB` `GiB` のように書けます。
  接頭辞の一覧は大きさ順（`10²⁴` 〜 `10⁻²⁴`、2進接頭辞も同列）で表示します。
- **次元を検査** — `km → lb` のような次元の違う変換はエラーにします。
- **温度は別扱い** — `degC` / `degF` / `K` をアフィン変換として正しく換算します。
- **読みやすい接頭辞で併記** — `0.0000035 m` を `mm` にすると `≈ 3.5 μm` も表示します。
- **計算はすべて Rust コア** — CLI と Web UI は同じロジックを共有します（C# は表示のみ）。
- **依存クレートゼロ** — Rust コアは外部クレートを使いません。

## 構成

| ディレクトリ | 中身 |
| --- | --- |
| `core/` | Rust の変換コア（`lib` + `cdylib`）。処理はすべてここ。 |
| `cli/` | Rust 製 CLI `uconv`。 |
| `web/` | C# / ASP.NET Core の Web UI。Rust コアを P/Invoke で呼ぶ薄いフロント。 |

## 必要なもの

- Rust ツールチェーン（`cargo`）
- .NET SDK 8（Web UI を使う場合）

## 使い方

### CLI

```sh
cd cli
cargo run --release -- 3.2 GHz MHz      # 3200 MHz  （≈ 3.2 GHz）
cargo run --release -- 100 degC degF    # 212 °F
cargo run --release -- scale 5 m p      # 5e9
```

ビルドして `target/release/uconv` を直接使うこともできます。詳しくは [cli/README.md](cli/README.md)。

### Web UI

```sh
cd web
dotnet run          # → http://127.0.0.1:5080
```

ブラウザで値を入れると即時に変換します。変換前と変換後を入れ替える ⇄ ボタン、
接頭辞の一覧（大きさ順）つき。`dotnet build` 時に Rust コアを
`cargo build --release` でビルドし、`libunitconv_core.so` を出力へコピーします。

### テスト

```sh
cargo test          # リポジトリのルートで
```

## CLI の使い方

```sh
uconv <値> <変換前> <変換後>      # 単位変換（接頭辞つき）
uconv scale <値> <A> <B>         # 接頭辞スケーリング（ミリ → ピコ など）
uconv base <値> <基数>           # 2 / 8 / 10 / 16 進数へ
uconv --prefixes                # 接頭辞の一覧（大きさ順）
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

## Web API

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

Rust のコアを C ABI（`cdylib`）として公開し、CLI は `lib` として、C# の Web UI は
P/Invoke で呼び出します。

```
core/ ──(lib)──▶ cli/ (uconv)
    └──(cdylib: libunitconv_core.so)──▶ web/ (C#, P/Invoke)
```

C ABI の関数は `unitconv_convert` / `unitconv_scale` / `unitconv_prefixes` /
`unitconv_units`（いずれも UTF-8 の JSON 文字列を返し、呼び出し側が
`unitconv_free` で解放します）。

変換の考え方:

1. トークンを「単位そのもの」として照合（`m`→メートル、`min`→分）
2. 一致しなければ「接頭辞＋単位」に分解（`GHz`→ G + Hz、`μm`→ μ + m）
3. 次元ベクトル（m, kg, s, A, K, mol, cd, bit）が一致するか確認して比率で換算
4. 温度だけは `scale` / `offset` を持つ別表現にし、K を経由して変換

## ライセンス

MIT（[LICENSE](LICENSE)）
