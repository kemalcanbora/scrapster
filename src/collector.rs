use std::time::Duration;
use tokio::{sync::broadcast, time};

use crate::metrics::SystemMetrics;
use crate::sys_reader::*;
#[cfg(feature = "pi")]
use crate::pi::{read_dht22, PiSensors};

pub struct MetricsCollector {
    interval: Duration,
}

impl MetricsCollector {
    pub fn new(interval: Duration) -> Self {
        Self { interval }
    }

    pub async fn start(&self) -> broadcast::Receiver<SystemMetrics> {
        let (tx, rx) = broadcast::channel(8);
        let interval = self.interval;

        tokio::spawn(async move {
            #[cfg(feature = "pi")]
            let mut pi_sensors = match PiSensors::new() {
                Ok(ps) => Some(ps),
                Err(e) => {
                    eprintln!("PiSensors init failed: {}", e);
                    None
                }
            };

            let mut prev_cpu_tot: u64 = 0;
            let mut prev_cpu_idle: u64 = 0;
            let mut prev_ctxt: u64 = 0;
            let mut prev_vm_minor: u64 = 0;
            let mut prev_vm_major: u64 = 0;
            let mut prev_disks = read_diskstats();
            let mut prev_net = read_netdev();

            let mut ticker = time::interval(interval);
            loop {
                ticker.tick().await;

                let (user, system, idle, iowait, irq, softirq, steal, total, ctxt) =
                    read_cpu_stat(prev_cpu_tot, prev_cpu_idle, prev_ctxt);
                prev_cpu_tot = total;
                prev_cpu_idle = idle;
                prev_ctxt = ctxt;

                let cpu_usage = 100f32 * (total.saturating_sub(idle) as f32 / total as f32);

                let (rq_len, la1, la5, la15) = read_loadavg();

                let cpu_temp = read_cpu_temp();

                let throttle = read_throttle_status();

                let (mem_total, mem_used) = read_meminfo();
                let (vm_minor, vm_major) = read_vmstat(prev_vm_minor, prev_vm_major);
                prev_vm_minor = vm_minor;
                prev_vm_major = vm_major;

                let uptime = read_uptime();

                let disks = read_diskstats();
                let (dread, dwrite) = diff_diskstats(&prev_disks, &disks, interval.as_secs_f64());
                prev_disks = disks;

                let net = read_netdev();
                let (rx_bps, tx_bps) = diff_netdev(&prev_net, &net, interval.as_secs_f64());
                prev_net = net;

                // Optional Pi sensor readings
                #[cfg(feature = "pi")]
                let (fan_rpm_opt, env_temp_opt, env_humidity_opt, pwm_duty_opt) = {
                    // pulses measured since last tick
                    let rpm = if let Some(ref ps) = pi_sensors {
                        let pulses = ps.take_pulses() as f64;
                        Some((pulses / 2.0) * (60.0 / interval.as_secs_f64()))
                    } else {
                        None
                    };

                    let (t, h) = read_dht22().map(|(t, h)| (Some(t), Some(h))).unwrap_or((None, None));

                    let duty = if let Some(ref ps) = pi_sensors {
                        ps.pwm_duty_percent().ok()
                    } else {
                        None
                    };

                    (rpm, t, h, duty)
                };

                #[cfg(not(feature = "pi"))]
                let (fan_rpm_opt, env_temp_opt, env_humidity_opt, pwm_duty_opt) = (None, None, None, None);

                let metrics = SystemMetrics {
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    cpu_usage_percent: cpu_usage,
                    cpu_user_percent: 100f32 * (user as f32 / total as f32),
                    cpu_system_percent: 100f32 * (system as f32 / total as f32),
                    cpu_idle_percent: 100f32 * (idle as f32 / total as f32),
                    cpu_iowait_percent: 100f32 * (iowait as f32 / total as f32),
                    cpu_irq_percent: 100f32 * (irq as f32 / total as f32),
                    cpu_softirq_percent: 100f32 * (softirq as f32 / total as f32),
                    cpu_steal_percent: 100f32 * (steal as f32 / total as f32),
                    run_queue_length: rq_len,
                    context_switches_per_sec: ctxt,
                    cpu_temp_celsius: cpu_temp,
                    throttle_status: throttle,
                    mem_total_bytes: mem_total,
                    mem_used_bytes: mem_used,
                    page_faults_minor_per_sec: vm_minor,
                    page_faults_major_per_sec: vm_major,
                    uptime_seconds: uptime,
                    load_avg_1: la1,
                    load_avg_5: la5,
                    load_avg_15: la15,
                    disk_read_bytes_per_sec: dread,
                    disk_write_bytes_per_sec: dwrite,
                    net_rx_bytes_per_sec: rx_bps,
                    net_tx_bytes_per_sec: tx_bps,
                    fan_rpm: fan_rpm_opt,
                    env_temp_celsius: env_temp_opt,
                    env_humidity_percent: env_humidity_opt,
                    pwm_duty_percent: pwm_duty_opt,
                };

                let _ = tx.send(metrics);
            }
        });

        rx
    }
}

