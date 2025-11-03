pub mod metrics;
pub mod collector;
pub mod sys_reader;

pub use metrics::SystemMetrics;
pub use collector::MetricsCollector;

use std::time::{Duration, UNIX_EPOCH};

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyModule};

fn metrics_to_pydict<'py>(py: Python<'py>, m: &SystemMetrics) -> pyo3::Bound<'py, PyDict> {
    let out = PyDict::new_bound(py);
    let ts = m
        .timestamp
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);

    out.set_item("timestamp", ts).unwrap();
    out.set_item("cpu_usage_percent", m.cpu_usage_percent).unwrap();
    out.set_item("cpu_user_percent", m.cpu_user_percent).unwrap();
    out.set_item("cpu_system_percent", m.cpu_system_percent).unwrap();
    out.set_item("cpu_idle_percent", m.cpu_idle_percent).unwrap();
    out.set_item("cpu_iowait_percent", m.cpu_iowait_percent).unwrap();
    out.set_item("cpu_irq_percent", m.cpu_irq_percent).unwrap();
    out.set_item("cpu_softirq_percent", m.cpu_softirq_percent).unwrap();
    out.set_item("cpu_steal_percent", m.cpu_steal_percent).unwrap();
    out.set_item("run_queue_length", m.run_queue_length).unwrap();
    out.set_item("context_switches_per_sec", m.context_switches_per_sec).unwrap();
    out.set_item("cpu_temp_celsius", m.cpu_temp_celsius).unwrap();
    out.set_item("throttle_status", m.throttle_status).unwrap();
    out.set_item("mem_total_bytes", m.mem_total_bytes).unwrap();
    out.set_item("mem_used_bytes", m.mem_used_bytes).unwrap();
    out.set_item("page_faults_minor_per_sec", m.page_faults_minor_per_sec).unwrap();
    out.set_item("page_faults_major_per_sec", m.page_faults_major_per_sec).unwrap();
    out.set_item("uptime_seconds", m.uptime_seconds).unwrap();
    out.set_item("load_avg_1", m.load_avg_1).unwrap();
    out.set_item("load_avg_5", m.load_avg_5).unwrap();
    out.set_item("load_avg_15", m.load_avg_15).unwrap();
    out.set_item("disk_read_bytes_per_sec", m.disk_read_bytes_per_sec).unwrap();
    out.set_item("disk_write_bytes_per_sec", m.disk_write_bytes_per_sec).unwrap();
    out.set_item("net_rx_bytes_per_sec", m.net_rx_bytes_per_sec).unwrap();
    out.set_item("net_tx_bytes_per_sec", m.net_tx_bytes_per_sec).unwrap();
    out
}

#[pyfunction]
fn get_metrics_once(py: Python, interval_ms: u64) -> PyResult<PyObject> {
    // Use a dedicated runtime to await one sample
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{e}")))?;

    let metrics = rt.block_on(async move {
        let collector = MetricsCollector::new(Duration::from_millis(interval_ms));
        let mut rx = collector.start().await;
        rx.recv().await.ok()
    });

    match metrics {
        Some(m) => Ok(metrics_to_pydict(py, &m).into_py(py)),
        None => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            "failed to collect metrics",
        )),
    }
}

#[pymodule]
fn scrapster(_py: Python, m: &pyo3::Bound<PyModule>) -> PyResult<()> {
    m.add_function(pyo3::wrap_pyfunction_bound!(get_metrics_once, m)?)?;
    Ok(())
}

