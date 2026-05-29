# Design Notes

## Goal

RustGauge should stay quiet in the Windows notification area and show useful system status only when needed.

The app is not a large dashboard. It is a small resident monitor with a readable tray tooltip and a compact visual signal in the tray icon.

## Principles

- Keep the core independent from any future settings UI.
- Start with reliable metrics first.
- Treat device-dependent metrics as best effort.
- Store settings in TOML.
- Use the tray menu for quick display toggles.
- Show unavailable values as `unsupported` instead of failing the app.
- Avoid copied visual assets; draw the tray icon in code.

## Layers

```text
src/
|-- main.rs          Entry point
|-- app.rs           Event loop and menu actions
|-- config.rs        Config loading and saving
|-- metrics/         CPU, memory, GPU, NPU, network collectors
|-- render.rs        Tooltip text and dynamic tray icon rendering
|-- autostart.rs     Windows startup registration
`-- tray.rs          Thin adapter around tray-icon
```

## Display Menu

The right-click menu has a `Display` submenu. Each metric can be toggled without opening a settings window.

Current metrics:

- CPU
- MEM
- GPU
- NPU
- Network

Changes are saved to `enabled_metrics` in the TOML config.

## Startup

The right-click menu has a `Start with Windows` toggle. It writes a per-user Run registry value under `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`, so it does not require administrator privileges.

## Metrics

CPU and memory are collected through `sysinfo`.

Network uses `sysinfo` interface deltas and is displayed as throughput. The icon uses a gentle logarithmic activity scale so low traffic can still influence the visual state.

GPU and NPU are best-effort Windows PDH metrics. On this machine, NPU appears as a compute-only LUID inside `GPU Engine`, so the collector splits GPU and NPU by grouping GPU Engine instances.

## Tray Icon

The tray icon is generated in Rust at runtime. It does not use copied RunCat artwork or external bitmap assets.

The current icon draws CPU, memory, GPU, and NPU horizontally. Each cell fills from the bottom based on the current value.

Network stays in the tooltip only because it is harder to read as a tiny icon meter.

## Later

- More icon styles
- Better high-DPI tuning
- Full settings window
- More detailed GPU/NPU detection
