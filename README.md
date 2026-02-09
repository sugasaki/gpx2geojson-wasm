# gpx2geojson-wasm

GPX ファイルを GeoJSON に変換する高速ライブラリ。Rust で実装し、WebAssembly にコンパイルすることで、ブラウザや React Native などの JavaScript 環境からそのまま利用できます。

## 特徴

- GPX 1.0 / 1.1 両対応（ネームスペースあり・なし両方）
- Waypoint / Route / Track すべての要素を変換
- `coordinateProperties.times` は [@tmcw/togeojson](https://github.com/tmcw/togeojson) 互換フォーマット
- 1 ポイントのみのトラックは Point Feature として出力
- WASM サイズ: 191KB raw / 77KB gzip

## 必要な環境

- [Rust](https://www.rust-lang.org/tools/install) (stable)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) (WASM ビルド時)
- [binaryen](https://github.com/WebAssembly/binaryen) (wasm-opt による追加最適化時、任意)

## セットアップ

```bash
git clone https://github.com/sugasaki/gpx2geojson-wasm.git
cd gpx2geojson-wasm
```

## テスト

### Rust テスト

```bash
# 全テスト実行（ユニット + インテグレーション + スナップショット）
cargo test

# インテグレーションテストのみ
cargo test --test integration_test

# スナップショットテストのみ
cargo test --test snapshot_test

# スナップショットの期待ファイルを生成/更新
UPDATE_SNAPSHOTS=1 cargo test --test snapshot_test
```

### ブラウザテスト

TypeScript ラッパーと WASM の動作をブラウザで確認できます。

```bash
# ビルド（WASM + TypeScript）
npm install
npm run build

# ローカルサーバー起動
npm run serve

# ブラウザで http://localhost:3000/test.html を開く
```

`test.html` は自動的にテストを実行し、以下を検証します：
- WASM の自動初期化
- Waypoint と Track の変換
- GeoJSON の構造と座標値

## WASM ビルド

```bash
# pkg/ ディレクトリに WASM パッケージを出力
wasm-pack build --target web --release

# wasm-opt で追加最適化（任意、別途 binaryen が必要）
wasm-opt -Os --enable-bulk-memory -o pkg/optimized.wasm pkg/gpx2geojson_wasm_bg.wasm
```

## 使い方（JavaScript）

```javascript
import init, { gpxToGeoJson, gpxToGeoJsonString } from './pkg/gpx2geojson_wasm.js';

await init();

const gpxString = `<?xml version="1.0"?>
<gpx version="1.1">
  <wpt lat="35.6762" lon="139.6503">
    <name>Tokyo</name>
  </wpt>
</gpx>`;

// JS オブジェクトとして取得
const geojson = gpxToGeoJson(gpxString);
console.log(geojson);

// JSON 文字列として取得
const geojsonString = gpxToGeoJsonString(gpxString);
console.log(geojsonString);
```

### オプション

```javascript
const geojson = gpxToGeoJson(gpxString, {
  includeElevation: true,      // 標高を3番目の座標値に含める（デフォルト: true）
  includeTime: true,           // coordinateProperties.times にタイムスタンプを含める（デフォルト: true）
  includeMetadata: true,       // name, desc 等を properties に含める（デフォルト: true）
  types: ["waypoint", "track"],// 変換する要素タイプを指定（デフォルト: 全て）
  joinTrackSegments: false,    // トラックセグメントを MultiLineString に結合（デフォルト: false）
});
```

## 出力例

入力 GPX:

```xml
<?xml version="1.0"?>
<gpx version="1.1">
  <wpt lat="35.6762" lon="139.6503">
    <name>Tokyo Tower</name>
    <ele>40.5</ele>
  </wpt>
</gpx>
```

出力 GeoJSON:

```json
{
  "type": "FeatureCollection",
  "features": [
    {
      "type": "Feature",
      "geometry": {
        "type": "Point",
        "coordinates": [139.6503, 35.6762, 40.5]
      },
      "properties": {
        "gpxType": "waypoint",
        "name": "Tokyo Tower",
        "ele": 40.5
      }
    }
  ]
}
```

## プロジェクト構成

```
src/
├── lib.rs          # WASM エントリポイント (#[wasm_bindgen] exports)
├── parser.rs       # quick-xml ストリーミング GPX パーサ
├── gpx_types.rs    # 内部データ構造体
├── converter.rs    # GPX → GeoJSON 変換
├── options.rs      # ConvertOptions
└── error.rs        # エラー型定義
tests/
├── integration_test.rs            # インテグレーションテスト
├── snapshot_test.rs               # スナップショットテスト
└── fixtures/
    ├── basic/                     # 基本的な GPX ファイル
    ├── tracks/                    # トラック関連
    ├── edge_cases/                # エッジケース
    ├── vendor/                    # ベンダー固有の拡張
    └── expected/                  # スナップショット期待値（自動生成）
```

## 技術スタック

| 用途 | ライブラリ |
|------|-----------|
| XML パーサ | quick-xml 0.39 |
| JS 連携 | wasm-bindgen 0.2 + serde-wasm-bindgen 0.6 |
| GeoJSON 構築 | geojson 0.24 + serde_json |
| ビルド | wasm-pack (`--target web`) |

## ライセンス

MIT
