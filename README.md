# RustGauge

A lightweight Windows tray monitor written in Rust.

[日本語 README](README.ja.md)

The app lives in the notification area and shows selected metrics in the tray tooltip. CPU and memory are collected through `sysinfo`. Network throughput is also collected through `sysinfo`. GPU and NPU are collected through Windows PDH counters when the machine exposes them.

## Features

- Tray resident app for Windows
- CPU and memory usage
- Network throughput
- Best-effort GPU usage via `\GPU Engine(*)\Utilization Percentage`
- Best-effort NPU usage inferred from compute-only `\GPU Engine(*)\Utilization Percentage` counters
- Dynamic task-manager-style tray icon
- Tooltip with selected metrics
- Right-click `Display` submenu with checkable metric items
- Right-click `Start with Windows` toggle for user-level startup registration
- Exit menu item

## Run

```powershell
cargo run --release
```

On first startup, the app creates a config file. During development, `rust-gauge.toml` in the repository root is preferred if it exists. Otherwise, the app uses the user config directory.

See [config/example.toml](config/example.toml).

## Config

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

Supported `enabled_metrics` values:

- `cpu`
- `memory`
- `gpu`
- `npu`
- `network`

GPU/NPU may show as `unsupported` on machines that do not expose those Windows performance counters.

## Tray Icon

The tray icon is generated in Rust at runtime instead of using copied bitmap assets. It draws CPU, memory, GPU, and NPU horizontally. Network stays in the tooltip only.

The colors are inspired by Windows Task Manager. Network stays in the tooltip only because it is harder to read as a tiny icon meter.

## Startup

`Start with Windows` registers the current executable in:

```text
HKCU\Software\Microsoft\Windows\CurrentVersion\Run
```

This is a per-user setting and does not require administrator privileges.

## Privacy

RustGauge only reads local system metrics and does not send telemetry or network requests.

The app reads:

- CPU usage
- Memory usage
- Local network interface throughput
- GPU/NPU utilization from Windows performance counters, when available
- Its own TOML configuration file
- The per-user Windows startup registry value when `Start with Windows` is used

RustGauge does not collect personal files, browser data, process lists, window titles, or keystrokes.

## Design

See [docs/design.md](docs/design.md).

## License

RustGauge is licensed under either of:

- MIT license ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

at your option.
