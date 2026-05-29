#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod app;
mod autostart;
mod config;
mod metrics;
mod render;
mod tray;

fn main() {
    if let Err(error) = app::run() {
        eprintln!("RustGauge failed: {error:?}");
        std::process::exit(1);
    }
}
