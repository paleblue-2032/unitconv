# unitconv

単位と SI 接頭辞（m, µ, p, k, M…）を**パッと変換**するツール。
小さな CLI 版（Rust）と Web 版（C#）が同居するモノレポ。

- `cli/` — Rust 製 CLI **uconv**。依存クレートゼロ、起動 1ms 程度。
- `web/` — C# / ASP.NET Core 製の Web UI。ブラウザで動く。

どちらも同じ考え方の変換エンジンを持つ:

1. トークンを「単位そのもの」として照合（`m`→メートル、`min`→分）
2. ダメなら「接頭辞＋単位」に分解（`GHz`→ G + Hz、`µm`→ µ + m）
3. 次元（m,kg,s,A,K,mol,cd,bit）が一致するか確認して比率で換算
4. 温度（`degC` / `degF` / `K`）だけはアフィン変換として別扱い
5. 結果は読みやすい接頭辞でも併記（`0.0000035 m` → `≈ 3.5 µm`）

## Web 版（C#）

```sh
cd web
dotnet run            # → http://127.0.0.1:5080
```

- `GET /api/convert?value=5&from=km&to=mile`
- `GET /api/scale?value=5&from=m&to=p`
- `GET /api/prefixes` / `GET /api/units`

`wwwroot/` の HTML/CSS/JS をそのまま配信する。

## CLI 版（Rust）

```sh
cd cli
cargo run --release -- 3.2 GHz MHz      # 3200 MHz  （≈ 3.2 GHz）
cargo run --release -- 100 degC degF    # 212 °F
cargo run --release -- scale 5 m p      # 5e9
cargo test
```

詳しくは [cli/README.md](cli/README.md)。

## 対応している単位（抜粋）

長さ `m mi ft in yd nmi au ly pc` / 質量 `g t lb oz u` / 時間 `s min h day yr` /
電気 `A C V ohm S F Wb T H` / エネルギー `J cal kcal Wh eV` / 仕事率 `W hp` /
圧力 `Pa bar atm mmHg psi` / 速さ `kph mph kn c` / 振動数 `Hz` /
面積 `ha acre` / 体積 `L gal` / 情報 `bit B`（2進接頭辞 `KiB` 等も） / 放射線 `Bq Gy Sv`

## ライセンス

MIT
