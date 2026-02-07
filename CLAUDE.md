# gpx2geojson-wasm

GPXファイルをGeoJSONに変換する高速ライブラリ。Rust → WebAssembly。

## プロジェクト構成

```
src/
├── lib.rs          # WASMエントリポイント (#[wasm_bindgen] exports)
├── parser.rs       # quick-xml 0.39 ストリーミングGPXパーサ
├── gpx_types.rs    # 内部データ構造体 (GpxData, GpxPoint, GpxRoute, GpxTrack)
├── converter.rs    # GPX → GeoJSON変換 (geojson crate + serde_json)
├── options.rs      # ConvertOptions (serde camelCase)
└── error.rs        # Gpx2GeoJsonError + JsValue変換
```

## 公開API

```typescript
gpxToGeoJson(gpxString: string, options?: ConvertOptions): FeatureCollection
gpxToGeoJsonString(gpxString: string, options?: ConvertOptions): string
```

## ビルド・テスト

```bash
# テスト
cargo test

# WASMビルド (pkg/ に出力)
wasm-pack build --target web --release

# wasm-opt で追加最適化 (別途 binaryen が必要)
wasm-opt -Os --enable-bulk-memory -o pkg/optimized.wasm pkg/gpx2geojson_wasm_bg.wasm
```

## 技術スタック

- **XMLパーサ**: quick-xml 0.39 (Reader + local_name() バイトマッチ、NsReader不使用)
- **JS連携**: wasm-bindgen 0.2.108 + serde-wasm-bindgen 0.6.5
- **GeoJSON**: geojson 0.24 + serde_json
- **ビルド**: wasm-pack --target web
- **WASMサイズ**: 191KB raw / 77KB gzip

## 開発ワークフロー

- 新しい作業を始める前に **GitHub Issue を作成** する
- 作業は必ず **feature ブランチ** を切ってから行う (ブランチ名: `issue-{番号}-{概要}`)
- main ブランチへの直接コミットは禁止
- 作業完了後は PR を作成する
- **PR のマージは GitHub 上でレビュー完了後に行う（自動マージ禁止）**

## 設計上の注意点

- GPX 1.0/1.1 両対応 (`local_name()` でネームスペース非依存)
- ネームスペースあり・なし両方のGPXファイルを処理可能
- extensions は `read_to_end()` でスキップ (将来パース対応予定)
- 1ポイントのみのトラックは Point Feature として出力
- coordinateProperties.times は @tmcw/togeojson 互換フォーマット
- wasm-pack 同梱の wasm-opt は古いため Cargo.toml で無効化済み。binaryen の wasm-opt を別途使用
