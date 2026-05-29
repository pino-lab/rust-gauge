# RustGauge

A lightweight Windows tray monitor written in Rust.

[日本語 README](README.ja.md)

RustGauge stays in the Windows notification area and shows selected system metrics in the tray icon and tooltip. It is designed for a small always-on status view, not a full dashboard.

## Features

- Tray resident app for Windows
- CPU and memory usage
- Network throughput
- Best-effort GPU usage via Windows PDH counters
- Best-effort NPU usage inferred from compute-only PDH counters
- Dynamic task-manager-style tray icon
- Tooltip with selected metrics
- Right-click menu for display toggles, startup registration, and exit
- No telemetry or external network requests

## Install

### Download a release

1. Open the [Releases](https://github.com/pino-lab/rust-gauge/releases) page.
2. Download the latest Windows zip.
3. Extract the zip anywhere you like.
4. Run `rust-gauge.exe`.

RustGauge runs in the notification area. If you do not see it immediately, check the hidden tray icons menu in the Windows taskbar.

### Build from source

Install Rust, then run:

```powershell
cargo build --release
```

The executable is created at:

```text
target\release\rust-gauge.exe
```

You can also run it directly from the repository:

```powershell
cargo run --release
```

## Usage

Start `rust-gauge.exe`. The tray icon updates automatically, and the tooltip shows the selected metrics.

Right-click the tray icon to open the menu:

- `Display`: choose which metrics appear in the tray status and tooltip.
- `Start with Windows`: register or unregister RustGauge for user-level startup.
- `Exit`: close RustGauge.

Display changes are saved automatically.

## Configuration

On first startup, RustGauge creates a config file in the user config directory.

You can also place `rust-gauge.toml` in the current working directory before starting RustGauge. If that file exists, RustGauge uses it instead of the user config file.

See [config/example.toml](config/example.toml):

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

`update_interval_ms` is clamped to a minimum of 250 ms.

## Startup

When `Start with Windows` is enabled, RustGauge registers the current executable in:

```text
HKCU\Software\Microsoft\Windows\CurrentVersion\Run
```

This is a per-user setting and does not require administrator privileges.

If you move `rust-gauge.exe` to another folder, disable and enable `Start with Windows` again so the startup entry points to the new path.

## Notes

GPU and NPU metrics depend on Windows performance counters exposed by the machine and driver. If they are not available, RustGauge shows them as `unsupported`.

Network is shown in the tooltip. The tray icon focuses on CPU, memory, GPU, and NPU because network throughput is harder to read as a tiny meter.

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

## Troubleshooting

- The app started but no window appears: RustGauge is a tray app. Check the notification area.
- GPU or NPU shows `unsupported`: the required Windows performance counters are not available on that machine.
- Startup does not work after moving the exe: toggle `Start with Windows` off and on again.
- Config changes do not appear: restart RustGauge after editing TOML by hand.

## Design

See [docs/design.md](docs/design.md).

## Maintainers

See [docs/release.md](docs/release.md) for the release checklist.

## License

RustGauge is licensed under either of:

- MIT license ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

at your option.
