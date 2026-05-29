use serde::{Deserialize, Serialize};
use sysinfo::{Networks, System};

#[cfg(target_os = "windows")]
mod windows_pdh;

#[cfg(target_os = "windows")]
use windows_pdh::WindowsPdhCollector;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MetricKind {
    Cpu,
    Memory,
    Gpu,
    Npu,
    Network,
}

#[derive(Debug, Clone)]
pub struct MetricSample {
    pub kind: MetricKind,
    pub label: &'static str,
    pub value_percent: Option<f32>,
    pub display_value: Option<String>,
    pub status: MetricStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricStatus {
    Ok,
    Unsupported,
}

#[derive(Debug, Clone)]
pub struct MetricSnapshot {
    pub samples: Vec<MetricSample>,
}

pub struct MetricCollector {
    system: System,
    networks: Networks,
    #[cfg(target_os = "windows")]
    pdh: WindowsPdhCollector,
}

impl MetricCollector {
    pub fn new() -> Self {
        let mut system = System::new();
        system.refresh_cpu();
        system.refresh_memory();

        let mut networks = Networks::new_with_refreshed_list();
        networks.refresh();

        Self {
            system,
            networks,
            #[cfg(target_os = "windows")]
            pdh: WindowsPdhCollector::new(),
        }
    }

    pub fn collect(&mut self, enabled_metrics: &[MetricKind]) -> MetricSnapshot {
        self.system.refresh_cpu();
        self.system.refresh_memory();
        self.networks.refresh();
        #[cfg(target_os = "windows")]
        self.pdh.refresh();

        let samples = enabled_metrics
            .iter()
            .map(|kind| self.collect_one(*kind))
            .collect();

        MetricSnapshot { samples }
    }

    fn collect_one(&mut self, kind: MetricKind) -> MetricSample {
        match kind {
            MetricKind::Cpu => MetricSample {
                kind,
                label: "CPU",
                value_percent: Some(self.system.global_cpu_info().cpu_usage()),
                display_value: None,
                status: MetricStatus::Ok,
            },
            MetricKind::Memory => {
                let total = self.system.total_memory();
                let used = self.system.used_memory();
                let value_percent = if total > 0 {
                    Some((used as f32 / total as f32) * 100.0)
                } else {
                    None
                };

                MetricSample {
                    kind,
                    label: "MEM",
                    value_percent,
                    display_value: None,
                    status: MetricStatus::Ok,
                }
            }
            MetricKind::Gpu => self.collect_gpu(),
            MetricKind::Npu => self.collect_npu(),
            MetricKind::Network => self.collect_network(),
        }
    }

    fn collect_gpu(&mut self) -> MetricSample {
        #[cfg(target_os = "windows")]
        if let Some(value) = self.pdh.gpu_usage_percent() {
            return percent_sample(MetricKind::Gpu, "GPU", value);
        }

        unsupported(MetricKind::Gpu, "GPU")
    }

    fn collect_npu(&mut self) -> MetricSample {
        #[cfg(target_os = "windows")]
        if let Some(value) = self.pdh.npu_usage_percent() {
            return percent_sample(MetricKind::Npu, "NPU", value);
        }

        unsupported(MetricKind::Npu, "NPU")
    }

    fn collect_network(&self) -> MetricSample {
        let bytes_per_interval = self
            .networks
            .values()
            .map(|network| network.received() + network.transmitted())
            .sum::<u64>();

        MetricSample {
            kind: MetricKind::Network,
            label: "NET",
            value_percent: Some(network_activity_percent(bytes_per_interval)),
            display_value: Some(format!("{}/s", format_bytes(bytes_per_interval))),
            status: MetricStatus::Ok,
        }
    }
}

fn unsupported(kind: MetricKind, label: &'static str) -> MetricSample {
    MetricSample {
        kind,
        label,
        value_percent: None,
        display_value: None,
        status: MetricStatus::Unsupported,
    }
}

fn percent_sample(kind: MetricKind, label: &'static str, value: f32) -> MetricSample {
    MetricSample {
        kind,
        label,
        value_percent: Some(value.clamp(0.0, 100.0)),
        display_value: None,
        status: MetricStatus::Ok,
    }
}

fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let bytes = bytes as f64;

    if bytes >= GB {
        format!("{:.1} GB", bytes / GB)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes / MB)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes / KB)
    } else {
        format!("{bytes:.0} B")
    }
}

fn network_activity_percent(bytes: u64) -> f32 {
    const TEN_MB: f32 = 10.0 * 1024.0 * 1024.0;

    if bytes == 0 {
        0.0
    } else {
        (((bytes as f32 + 1.0).log10() / (TEN_MB + 1.0).log10()) * 100.0).clamp(0.0, 100.0)
    }
}
