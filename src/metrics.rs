#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub timestamp: std::time::SystemTime,
    // CPU
    pub cpu_usage_percent: f32,
    pub cpu_user_percent: f32,
    pub cpu_system_percent: f32,
    pub cpu_idle_percent: f32,
    pub cpu_iowait_percent: f32,
    pub cpu_irq_percent: f32,
    pub cpu_softirq_percent: f32,
    pub cpu_steal_percent: f32,
    pub run_queue_length: f32,
    pub context_switches_per_sec: u64,
    // Temperature / Throttling
    pub cpu_temp_celsius: f32,
    pub throttle_status: u32,
    // Memory
    pub mem_total_bytes: u64,
    pub mem_used_bytes: u64,
    pub page_faults_minor_per_sec: u64,
    pub page_faults_major_per_sec: u64,
    // Uptime / Load
    pub uptime_seconds: f64,
    pub load_avg_1: f32,
    pub load_avg_5: f32,
    pub load_avg_15: f32,
    // Disk
    pub disk_read_bytes_per_sec: u64,
    pub disk_write_bytes_per_sec: u64,
    // Network
    pub net_rx_bytes_per_sec: u64,
    pub net_tx_bytes_per_sec: u64,
}

