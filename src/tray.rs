use anyhow::{Context, Result};
use tray_icon::{
    menu::{CheckMenuItem, Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem, Submenu},
    TrayIcon, TrayIconBuilder,
};

use crate::{
    autostart,
    config::AppConfig,
    metrics::{MetricKind, MetricSnapshot},
    render::{render_icon, tooltip_text},
};

pub struct TrayController {
    tray_icon: TrayIcon,
    cpu_id: MenuId,
    memory_id: MenuId,
    gpu_id: MenuId,
    npu_id: MenuId,
    network_id: MenuId,
    autostart_id: MenuId,
    exit_id: MenuId,
    cpu_item: CheckMenuItem,
    memory_item: CheckMenuItem,
    gpu_item: CheckMenuItem,
    npu_item: CheckMenuItem,
    network_item: CheckMenuItem,
    autostart_item: CheckMenuItem,
    _display_menu: Submenu,
    _menu: Menu,
}

impl TrayController {
    pub fn new(config: &AppConfig) -> Result<Self> {
        let menu = Menu::new();
        let display_menu = Submenu::new("Display", true);
        let cpu_item = CheckMenuItem::new(
            "CPU",
            true,
            config.enabled_metrics.contains(&MetricKind::Cpu),
            None,
        );
        let memory_item = CheckMenuItem::new(
            "MEM",
            true,
            config.enabled_metrics.contains(&MetricKind::Memory),
            None,
        );
        let gpu_item = CheckMenuItem::new(
            "GPU",
            true,
            config.enabled_metrics.contains(&MetricKind::Gpu),
            None,
        );
        let npu_item = CheckMenuItem::new(
            "NPU",
            true,
            config.enabled_metrics.contains(&MetricKind::Npu),
            None,
        );
        let network_item = CheckMenuItem::new(
            "Network",
            true,
            config.enabled_metrics.contains(&MetricKind::Network),
            None,
        );
        let separator = PredefinedMenuItem::separator();
        let autostart_item = CheckMenuItem::new(
            "Start with Windows",
            true,
            autostart::is_enabled().unwrap_or(false),
            None,
        );
        let autostart_separator = PredefinedMenuItem::separator();
        let exit_item = MenuItem::new("Exit", true, None);

        display_menu
            .append_items(&[
                &cpu_item,
                &memory_item,
                &PredefinedMenuItem::separator(),
                &gpu_item,
                &npu_item,
                &network_item,
            ])
            .context("failed to create display submenu")?;

        let cpu_id = cpu_item.id().clone();
        let memory_id = memory_item.id().clone();
        let gpu_id = gpu_item.id().clone();
        let npu_id = npu_item.id().clone();
        let network_id = network_item.id().clone();
        let autostart_id = autostart_item.id().clone();
        let exit_id = exit_item.id().clone();

        menu.append_items(&[
            &display_menu,
            &separator,
            &autostart_item,
            &autostart_separator,
            &exit_item,
        ])
        .context("failed to create tray menu")?;

        let empty_snapshot = MetricSnapshot { samples: vec![] };
        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu.clone()))
            .with_menu_on_left_click(false)
            .with_menu_on_right_click(true)
            .with_tooltip(config.app_name.as_str())
            .with_icon(render_icon(&empty_snapshot, config)?)
            .build()
            .context("failed to create tray icon")?;

        Ok(Self {
            tray_icon,
            cpu_id,
            memory_id,
            gpu_id,
            npu_id,
            network_id,
            autostart_id,
            exit_id,
            cpu_item,
            memory_item,
            gpu_item,
            npu_item,
            network_item,
            autostart_item,
            _display_menu: display_menu,
            _menu: menu,
        })
    }

    pub fn update(&mut self, snapshot: &MetricSnapshot, config: &AppConfig) -> Result<()> {
        self.tray_icon
            .set_tooltip(Some(tooltip_text(snapshot, config)))
            .context("failed to update tray tooltip")?;
        self.tray_icon
            .set_icon(Some(render_icon(snapshot, config)?))
            .context("failed to update tray icon")?;
        Ok(())
    }

    pub fn show_status(&mut self, message: &str) -> Result<()> {
        self.tray_icon
            .set_tooltip(Some(message))
            .context("failed to show tray status")
    }

    pub fn show_error(&mut self, message: &str) -> Result<()> {
        self.tray_icon
            .set_tooltip(Some(message))
            .context("failed to show tray error")
    }

    pub fn is_display_metric(&self, event: &MenuEvent) -> bool {
        event.id == self.cpu_id
            || event.id == self.memory_id
            || event.id == self.gpu_id
            || event.id == self.npu_id
            || event.id == self.network_id
    }

    pub fn enabled_metrics_from_menu(&self) -> Vec<MetricKind> {
        let mut enabled_metrics = Vec::new();

        if self.cpu_item.is_checked() {
            enabled_metrics.push(MetricKind::Cpu);
        }

        if self.memory_item.is_checked() {
            enabled_metrics.push(MetricKind::Memory);
        }

        if self.gpu_item.is_checked() {
            enabled_metrics.push(MetricKind::Gpu);
        }

        if self.npu_item.is_checked() {
            enabled_metrics.push(MetricKind::Npu);
        }

        if self.network_item.is_checked() {
            enabled_metrics.push(MetricKind::Network);
        }

        enabled_metrics
    }

    pub fn is_autostart(&self, event: &MenuEvent) -> bool {
        event.id == self.autostart_id
    }

    pub fn autostart_enabled_from_menu(&self) -> bool {
        self.autostart_item.is_checked()
    }

    pub fn set_autostart_checked(&self, enabled: bool) {
        self.autostart_item.set_checked(enabled);
    }

    pub fn is_exit(&self, event: &MenuEvent) -> bool {
        event.id == self.exit_id
    }
}
