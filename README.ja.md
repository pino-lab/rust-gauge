# RustGauge

Rust で作った軽量な Windows タスクトレイ監視ツールです。

[English README](README.md)

RustGauge は Windows の通知領域に常駐し、選択したシステムメトリクスをトレイアイコンとツールチップで表示します。大きなダッシュボードではなく、常に邪魔にならない小さな状態表示を目的にしています。

## 機能

- Windows の通知領域に常駐
- CPU / メモリ使用率
- ネットワーク通信量
- Windows PDH カウンターによる GPU 使用率の取得
- compute-only の PDH カウンターをもとにした NPU 使用率の推定取得
- タスクマネージャー風の動的トレイアイコン
- 選択したメトリクスを表示するツールチップ
- 右クリックメニューで表示項目、自動起動、終了を操作
- テレメトリ送信や外部ネットワーク通信なし

## インストール

### リリース版を使う

1. [Releases](https://github.com/pino-lab/rust-gauge/releases) を開きます。
2. 最新の Windows 用 zip をダウンロードします。
3. zip を好きな場所に展開します。
4. `rust-gauge.exe` を実行します。

RustGauge は通知領域で動作します。すぐに見つからない場合は、Windows タスクバーの隠れているインジケーターも確認してください。

### ソースからビルドする

Rust をインストールしてから、次を実行します。

```powershell
cargo build --release
```

実行ファイルは次の場所に作成されます。

```text
target\release\rust-gauge.exe
```

リポジトリから直接起動する場合は、次を実行します。

```powershell
cargo run --release
```

## 使い方

`rust-gauge.exe` を起動します。トレイアイコンは自動で更新され、ツールチップに選択中のメトリクスが表示されます。

トレイアイコンを右クリックするとメニューを開けます。

- `Display`: 表示するメトリクスを選択します。
- `Start with Windows`: Windows 起動時に RustGauge を自動起動するか切り替えます。
- `Exit`: RustGauge を終了します。

表示項目の変更は自動で保存されます。

## 設定

初回起動時にユーザー設定ディレクトリへ設定ファイルを作成します。

起動前にカレントディレクトリへ `rust-gauge.toml` を置いた場合は、そのファイルを優先して使います。

設定例は [config/example.toml](config/example.toml) を参照してください。

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

`update_interval_ms` は最小 250 ms に丸められます。

## 自動起動

`Start with Windows` を有効にすると、現在の実行ファイルを次の場所に登録します。

```text
HKCU\Software\Microsoft\Windows\CurrentVersion\Run
```

これはユーザー単位の設定で、管理者権限は不要です。

`rust-gauge.exe` を別のフォルダへ移動した場合は、`Start with Windows` を一度オフにしてから再度オンにしてください。新しい場所で登録し直されます。

## 補足

GPU / NPU メトリクスは、端末やドライバーが公開している Windows パフォーマンスカウンターに依存します。利用できない場合は `unsupported` と表示されます。

ネットワーク通信量はツールチップに表示されます。小さなトレイアイコンでは読み取りにくいため、アイコン表示は CPU、メモリ、GPU、NPU を中心にしています。

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

## トラブルシュート

- 起動してもウィンドウが出ない: RustGauge はトレイアプリです。通知領域を確認してください。
- GPU / NPU が `unsupported` になる: 対象の Windows パフォーマンスカウンターが利用できない環境です。
- exe を移動したあと自動起動しない: `Start with Windows` を一度オフにしてから再度オンにしてください。
- TOML を編集しても反映されない: 手で設定を変更したあとは RustGauge を再起動してください。

## 設計

[docs/design.md](docs/design.md) を参照してください。

## メンテナ向け

リリース手順は [docs/release.md](docs/release.md) を参照してください。

## ライセンス

RustGauge は以下のいずれかのライセンスで利用できます。

- MIT license ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

利用者がどちらかを選択できます。
