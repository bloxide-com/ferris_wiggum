use std::time::Duration;

use sysinfo::System;
use tokio::sync::broadcast;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryStatus {
    Ok,
    Warning,
    Critical,
    Unknown,
}

#[derive(Debug, Clone, Copy)]
pub struct MemorySnapshot {
    /// Total system memory (KiB)
    pub total_kib: u64,
    /// Used system memory (KiB)
    pub used_kib: u64,
    /// Used percent (0.0 - 100.0)
    pub percent_used: f64,
}

pub struct MemoryMonitor {
    system: System,
    warn_threshold_percent_used: f64,
    critical_threshold_percent_used: f64,
}

impl MemoryMonitor {
    pub fn new(warn_percent_used: f64, critical_percent_used: f64) -> Self {
        Self {
            system: System::new(),
            warn_threshold_percent_used: warn_percent_used,
            critical_threshold_percent_used: critical_percent_used,
        }
    }

    pub fn snapshot(&mut self) -> (MemoryStatus, Option<MemorySnapshot>) {
        self.system.refresh_memory();
        let total = self.system.total_memory();
        let used = self.system.used_memory();

        if total == 0 {
            return (MemoryStatus::Unknown, None);
        }

        let percent_used = (used as f64 / total as f64) * 100.0;
        let snapshot = MemorySnapshot {
            total_kib: total,
            used_kib: used,
            percent_used,
        };

        if percent_used >= self.critical_threshold_percent_used {
            (MemoryStatus::Critical, Some(snapshot))
        } else if percent_used >= self.warn_threshold_percent_used {
            (MemoryStatus::Warning, Some(snapshot))
        } else {
            (MemoryStatus::Ok, Some(snapshot))
        }
    }

    pub fn check_and_log(&mut self) -> MemoryStatus {
        let (status, snapshot) = self.snapshot();

        if let Some(s) = snapshot {
            let used_mib = s.used_kib / 1024;
            let total_mib = s.total_kib / 1024;

            match status {
                MemoryStatus::Critical => {
                    tracing::error!(
                        "CRITICAL: Memory usage at {:.1}% ({} MiB / {} MiB)",
                        s.percent_used,
                        used_mib,
                        total_mib
                    );
                }
                MemoryStatus::Warning => {
                    tracing::warn!(
                        "WARNING: Memory usage at {:.1}% ({} MiB / {} MiB)",
                        s.percent_used,
                        used_mib,
                        total_mib
                    );
                }
                MemoryStatus::Ok => {}
                MemoryStatus::Unknown => {
                    tracing::warn!("WARNING: Unable to read system memory usage");
                }
            }
        } else if status == MemoryStatus::Unknown {
            tracing::warn!("WARNING: Unable to read system memory usage");
        }

        status
    }
}

pub async fn run_memory_monitor(
    interval: Duration,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    let mut monitor = MemoryMonitor::new(80.0, 95.0);
    let mut ticker = tokio::time::interval(interval);

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                monitor.check_and_log();
            }
            _ = shutdown_rx.recv() => {
                tracing::info!("Memory monitor shutting down");
                break;
            }
        }
    }
}

