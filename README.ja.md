# RustGauge

Rust で作った軽量な Windows タスクトレイ監視ツールです。

[English README](README.md)

RustGauge は通知領域に常駐し、トレイアイコンとツールチップで選択したメトリクスを表示します。CPU とメモリ、ネットワーク通信量は `sysinfo` から取得します。GPU と NPU は、端末が公開している Windows PDH カウンターから取得します。

## 機能

- Windows の通知領域に常駐
- CPU / MEM 使用率
- ネットワーク通信量
- `\GPU Engine(*)\Utilization Percentage` による GPU 使用率の取得
- `\GPU Engine(*)\Utilization Percentage` をもとにした NPU 使用率の推定取得
- Rust で実行時生成するタスクマネージャー風トレイアイコン
- 選択した項目を表示するツールチップ
- 右クリックの `Display` サブメニューで表示項目を切り替え
- 右クリックの `Start with Windows` でユーザー単位の自動起動を切り替え
- `Exit` メニュー

## 実行

```powershell
cargo run --release
```

初回起動時に設定ファイルを作成します。開発中は、リポジトリ直下に `rust-gauge.toml` があればそれを優先します。無い場合はユーザー設定ディレクトリの設定ファイルを使います。

設定例は [config/example.toml](config/example.toml) を参照してください。

## 設定

```toml
app_name = "RustGauge"
update_interval_ms = 1000
enabled_metrics = ["cpu", "memory", "gpu", "npu", "network"]

[icon]
size = 32
low_color = "#48c774"
medium_color = "#ffdd57"
high_color = "#f14668"
track_color = "#2b2f36"

[thresholds]
medium = 60.0
high = 85.0
```

`enabled_metrics` で使える値:

- `cpu`
- `memory`
- `gpu`
- `npu`
- `network`

GPU / NPU は端末やドライバーが対象の Windows パフォーマンスカウンターを公開していない場合、`unsupported` と表示されることがあります。

## 自動起動

`Start with Windows` を有効にすると、現在の実行ファイルを次の場所に登録します。

```text
HKCU\Software\Microsoft\Windows\CurrentVersion\Run
```

これはユーザー単位の設定で、管理者権限は不要です。

## プライバシー

RustGauge はローカルのシステムメトリクスだけを読み取り、テレメトリ送信や外部ネットワーク通信は行いません。

読み取る情報:

- CPU 使用率
- メモリ使用率
- ローカルネットワークインターフェイスの通信量
- 利用可能な場合のみ、Windows パフォーマンスカウンターの GPU / NPU 使用率
- 自身の TOML 設定ファイル
- `Start with Windows` 利用時のユーザー単位 Windows 自動起動レジストリ値

RustGauge は個人ファイル、ブラウザーデータ、プロセス一覧、ウィンドウタイトル、キー入力を収集しません。

## 設計

[docs/design.md](docs/design.md) を参照してください。

## ライセンス

RustGauge は以下のいずれかのライセンスで利用できます。

- MIT license ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

利用者がどちらかを選択できます。
