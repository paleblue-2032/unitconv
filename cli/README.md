# uconv

[unitconv](../README.md) のコアを使った CLI。単位と SI 接頭辞を爆速で変換します。
同じコアを Web UI（C#）も使うので、変換結果は CLI と完全に一致します。

- **依存クレートゼロ**（コアは外部クレートを使いません）。起動は 1ms 程度。
- SI 接頭辞をそのまま扱う（`km` `GHz` `μm` `ms` `kB` `GiB` …）
- 次元（長さ・質量・時間…）を検査して、`km → lb` のような変換は弾く
- 温度（`degC` / `degF` / `K`）はアフィン変換として別扱い
- 結果は読みやすい接頭辞でも併記（`0.0000035 m` → `≈ 3.5 μm`）

## ビルド

```sh
cd cli
cargo build --release      # バイナリは ../target/release/uconv（ワークスペース）
cargo test                 # 動作テスト
```

ワークスペース全体をビルドするときは、リポジトリのルートで `cargo build --release`。

## 使い方

```sh
uconv 5 km mile          # 3.1068559612 mile
uconv 3.2 GHz MHz        # 3200 MHz  （≈ 3.2 GHz）
uconv 0.0000035 m mm     # 0.0035 mm  （≈ 3.5 μm）
uconv 100 degC degF      # 212 °F
uconv 98.6 degF K        # 310.15 K
uconv 1 GiB MiB          # 1024 MiB
uconv 65 mph kph         # 104.60736 kph
uconv 1 eV J             # 1.602176634e-19 J
uconv 1/2 m mm           # 分数もOK
```

そのほか:

```sh
uconv scale 5 m p        # 接頭辞スケーリング（5 ミリ → ピコ = 5e9）
uconv base ff 16         # 進数変換（2 / 8 / 10 / 16 を一気に）
uconv --prefixes         # 接頭辞の一覧（大きさ順・micro は μ で表示）
uconv --units            # 単位の一覧
uconv                    # 引数なしで対話モード
```

対話モード:

```
$ uconv
> 5 km mile
3.1068559612 mile
> scale 3 k M
0.003
> quit
```

## 温度の注意

`C` はクーロン、`F` はファラド、`K` はケルビンです。温度として使うときは
**`degC` / `degF` / `K`**（`°C` / `°F` も可）と書いてください。

| 書き方 | 意味 |
| --- | --- |
| `degC` / `°C` / `celsius` | 摂氏 |
| `degF` / `°F` / `fahrenheit` | 華氏 |
| `K` / `kelvin` | 絶対温度 |

## 対応している単位（抜粋）

長さ `m mi ft in yd nmi Å au ly pc` / 質量 `g t lb oz u(amu)` /
時間 `s min h day wk yr` / 電気 `A C V ohm S F Wb T H` /
エネルギー `J cal kcal Wh eV` / 仕事率 `W hp` / 圧力 `Pa bar atm mmHg psi` /
速さ `kph mph kn c` / 振動数 `Hz` / 面積 `ha acre` / 体積 `L gal` /
情報 `bit B`（`KiB` などの2進接頭辞も） / 放射線 `Bq Gy Sv` …ほか。

`uconv --units` で全一覧を表示します。

## 仕組み

1. トークンを「単位そのもの」として照合（`m`→メートル、`min`→分）
2. 一致しなければ「接頭辞＋単位」に分解（`GHz`→ G + Hz、`μm`→ μ + m）
3. 次元ベクトル（m, kg, s, A, K, mol, cd, bit）が一致するか確認して比率で換算
4. 温度だけは `scale` / `offset` を持つ別表現にし、K を経由して変換

変換ロジックはコア（`../core`）にあり、Web UI と共有しています。
