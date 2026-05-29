use std::{
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};

use anyhow::Result;
use tao::{
    event::Event,
    event_loop::{ControlFlow, EventLoopBuilder},
};
use tray_icon::menu::MenuEvent;

use crate::{
    autostart,
    config::{load_or_create_config, save_config_to_path, AppConfig},
    metrics::{MetricCollector, MetricSnapshot},
    tray::TrayController,
};

enum UserEvent {
    Metrics(MetricSnapshot),
    Menu(MenuEvent),
}

pub fn run() -> Result<()> {
    let loaded_config = load_or_create_config()?;
    let config_path = loaded_config.path;
    let config = Arc::new(RwLock::new(loaded_config.config));

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

    let menu_proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = menu_proxy.send_event(UserEvent::Menu(event));
    }));

    let sampling_proxy = event_loop.create_proxy();
    spawn_sampler(Arc::clone(&config), sampling_proxy);

    let initial_config = config.read().expect("config lock poisoned").clone();
    let mut tray = TrayController::new(&initial_config)?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(UserEvent::Metrics(snapshot)) => {
                let current_config = config.read().expect("config lock poisoned").clone();
                if let Err(error) = tray.update(&snapshot, &current_config) {
                    let _ = tray.show_error(&format!("Update failed: {error}"));
                }
            }
            Event::UserEvent(UserEvent::Menu(event)) => {
                if tray.is_display_metric(&event) {
                    let mut new_config = config.read().expect("config lock poisoned").clone();
                    new_config.enabled_metrics = tray.enabled_metrics_from_menu();

                    match save_config_to_path(&config_path, &new_config) {
                        Ok(()) => {
                            *config.write().expect("config lock poisoned") = new_config;
                            let _ = tray.show_status("Display settings saved");
                        }
                        Err(error) => {
                            let _ = tray.show_error(&format!("Config save failed: {error}"));
                        }
                    }
                } else if tray.is_autostart(&event) {
                    let enabled = tray.autostart_enabled_from_menu();

                    match autostart::set_enabled(enabled) {
                        Ok(()) => {
                            let status = if enabled {
                                "Startup enabled"
                            } else {
                                "Startup disabled"
                            };
                            let _ = tray.show_status(status);
                        }
                        Err(error) => {
                            tray.set_autostart_checked(!enabled);
                            let _ = tray.show_error(&format!("Startup update failed: {error}"));
                        }
                    }
                } else if tray.is_exit(&event) {
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => {}
        }
    });
}

fn spawn_sampler(
    config: Arc<RwLock<AppConfig>>,
    proxy: tao::event_loop::EventLoopProxy<UserEvent>,
) {
    thread::spawn(move || {
        let mut collector = MetricCollector::new();

        loop {
            let current_config = config.read().expect("config lock poisoned").clone();
            let snapshot = collector.collect(&current_config.enabled_metrics);

            if proxy.send_event(UserEvent::Metrics(snapshot)).is_err() {
                break;
            }

            thread::sleep(Duration::from_millis(
                current_config.update_interval_ms.max(250),
            ));
        }
    });
}
