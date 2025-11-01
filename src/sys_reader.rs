use std::collections::HashMap;

pub fn read_cpu_stat(_prev_total: u64, _prev_idle: u64, prev_ctxt: u64) -> (u64, u64, u64, u64, u64, u64, u64, u64, u64) {
    let stat = std::fs::read_to_string("/proc/stat").unwrap_or_default();
    let mut user: u64 = 0;
    let mut nice: u64 = 0;
    let mut system: u64 = 0;
    let mut idle: u64 = 0;
    let mut iowait: u64 = 0;
    let mut irq: u64 = 0;
    let mut softirq: u64 = 0;
    let mut steal: u64 = 0;
    let mut ctxt: u64 = 0;

    for line in stat.lines() {
        if line.starts_with("cpu ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 8 {
                user = parts[1].parse().unwrap_or(0);
                nice = parts[2].parse().unwrap_or(0);
                system = parts[3].parse().unwrap_or(0);
                idle = parts[4].parse().unwrap_or(0);
                iowait = parts[5].parse().unwrap_or(0);
                irq = parts[6].parse().unwrap_or(0);
                softirq = parts[7].parse().unwrap_or(0);
                if parts.len() >= 9 {
                    steal = parts[8].parse().unwrap_or(0);
                }
            }
        } else if line.starts_with("ctxt ") {
            ctxt = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
        }
    }

    let total = user + nice + system + idle + iowait + irq + softirq + steal;
    let delta_ctxt = ctxt.saturating_sub(prev_ctxt);
    (user + nice, system, idle, iowait, irq, softirq, steal, total, delta_ctxt)
}

pub fn read_loadavg() -> (f32, f32, f32, f32) {
    if let Ok(content) = std::fs::read_to_string("/proc/loadavg") {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 5 {
            let la1 = parts[0].parse().unwrap_or(0.0);
            let la5 = parts[1].parse().unwrap_or(0.0);
            let la15 = parts[2].parse().unwrap_or(0.0);
            let rq_info = parts[3];
            let rq_parts: Vec<&str> = rq_info.split('/').collect();
            if let Some(running) = rq_parts.get(0) {
                if let Ok(r) = running.parse::<f32>() {
                    return (r, la1, la5, la15);
                }
            }
            return (0.0, la1, la5, la15);
        }
    }
    (0.0, 0.0, 0.0, 0.0)
}

pub fn read_cpu_temp() -> f32 {
    for entry in glob::glob("/sys/class/thermal/thermal_zone*/temp").unwrap() {
        if let Ok(path) = entry {
            if let Ok(data) = std::fs::read_to_string(path) {
                if let Ok(val) = data.trim().parse::<f32>() {
                    return val / 1000.0;
                }
            }
        }
    }
    0.0
}

pub fn read_throttle_status() -> u32 {
    if let Ok(output) = std::process::Command::new("vcgencmd")
        .arg("get_throttled")
        .output()
    {
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout);
            if let Some(idx) = s.find("0x") {
                if let Ok(val) = u32::from_str_radix(&s[idx+2..].trim(), 16) {
                    return val;
                }
            }
        }
    }
    0
}

pub fn read_meminfo() -> (u64, u64) {
    let mut total: u64 = 0;
    let mut free: u64 = 0;
    let mut buffers: u64 = 0;
    let mut cached: u64 = 0;
    if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
        for line in content.lines() {
            let mut parts = line.split_whitespace();
            match parts.next().unwrap_or("") {
                "MemTotal:" => total = parts.next().unwrap_or("0").parse().unwrap_or(0),
                "MemFree:"  => free = parts.next().unwrap_or("0").parse().unwrap_or(0),
                "Buffers:"  => buffers = parts.next().unwrap_or("0").parse().unwrap_or(0),
                "Cached:"   => cached = parts.next().unwrap_or("0").parse().unwrap_or(0),
                _ => {}
            }
        }
    }
    let used = total.saturating_sub(free + buffers + cached);
    (total * 1024, used * 1024)
}

pub fn read_vmstat(prev_minor: u64, prev_major: u64) -> (u64, u64) {
    let mut minor = 0u64;
    let mut major = 0u64;
    if let Ok(content) = std::fs::read_to_string("/proc/vmstat") {
        for line in content.lines() {
            if line.starts_with("pgfault ") {
                minor = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
            }
            if line.starts_with("pgmajfault ") {
                major = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
            }
        }
    }
    (minor.saturating_sub(prev_minor), major.saturating_sub(prev_major))
}

pub fn read_uptime() -> f64 {
    std::fs::read_to_string("/proc/uptime")
        .ok()
        .and_then(|s| s.split_whitespace().next()?.parse().ok())
        .unwrap_or(0.0)
}

pub fn read_diskstats() -> HashMap<String, (u64, u64)> {
    let mut result = HashMap::new();
    if let Ok(content) = std::fs::read_to_string("/proc/diskstats") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 14 { continue; }
            let name = parts[2].to_string();
            let reads = parts[5].parse::<u64>().unwrap_or(0);
            let writes = parts[9].parse::<u64>().unwrap_or(0);
            result.insert(name, (reads * 512, writes * 512));
        }
    }
    result
}

pub fn diff_diskstats(prev: &HashMap<String, (u64, u64)>, curr: &HashMap<String, (u64, u64)>, interval: f64) -> (u64, u64) {
    let mut read_delta = 0u64;
    let mut write_delta = 0u64;
    for (dev, &(r, w)) in curr {
        if let Some(&(pr, pw)) = prev.get(dev) {
            read_delta += r.saturating_sub(pr);
            write_delta += w.saturating_sub(pw);
        }
    }
    ((read_delta as f64 / interval) as u64, (write_delta as f64 / interval) as u64)
}

pub fn read_netdev() -> HashMap<String, (u64, u64)> {
    let mut result = HashMap::new();
    if let Ok(content) = std::fs::read_to_string("/proc/net/dev") {
        for line in content.lines().skip(2) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 17 { continue; }
            let iface = parts[0].trim_end_matches(':').to_string();
            let rx = parts[1].parse::<u64>().unwrap_or(0);
            let tx = parts[9].parse::<u64>().unwrap_or(0);
            result.insert(iface, (rx, tx));
        }
    }
    result
}

pub fn diff_netdev(prev: &HashMap<String, (u64, u64)>, curr: &HashMap<String, (u64, u64)>, interval: f64) -> (u64, u64) {
    let mut rx_delta = 0u64;
    let mut tx_delta = 0u64;
    for (iface, &(r, t)) in curr {
        if let Some(&(pr, pt)) = prev.get(iface) {
            rx_delta += r.saturating_sub(pr);
            tx_delta += t.saturating_sub(pt);
        }
    }
    ((rx_delta as f64 / interval) as u64, (tx_delta as f64 / interval) as u64)
}

