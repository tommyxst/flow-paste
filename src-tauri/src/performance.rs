use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use serde::Serialize;
use sysinfo::{Pid, System};
use tauri::{AppHandle, Manager, State};

use crate::privacy::scan_pii;

/// Performance state manager
#[derive(Default)]
pub struct PerfState {
    pub latency_start: Mutex<Option<Instant>>,
    pub samples: Mutex<Vec<PerfSample>>,
}

/// Unified performance sample structure (Codex design)
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PerfSample {
    pub name: String,
    pub duration_ms: u64,
    pub tags: HashMap<String, String>,
    pub timestamp: u64,
}

/// Performance report for output
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PerfReport {
    pub meta: ReportMeta,
    pub metrics: Vec<PerfMetric>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportMeta {
    pub timestamp: u64,
    pub platform: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PerfMetric {
    pub name: String,
    pub value_ms: Option<u64>,
    pub value_mb: Option<u64>,
    pub target: u64,
    pub status: String,
}

impl PerfState {
    /// Start timing for hotkey trigger (called from hotkey module)
    pub fn start_timer(&self) {
        *self.latency_start.lock().unwrap() = Some(Instant::now());
        log::debug!("PERF: Timer started for hotkey trigger");
    }

    /// Add a performance sample to the collection
    fn add_sample(&self, sample: PerfSample) {
        self.samples.lock().unwrap().push(sample);
    }
}

/// Report render timestamp from frontend (Tauri command)
#[tauri::command]
pub fn report_render_timestamp(state: State<PerfState>) {
    let mut start_opt = state.latency_start.lock().unwrap();
    if let Some(start) = *start_opt {
        let duration = start.elapsed();
        let duration_ms = duration.as_millis() as u64;

        log::info!("PERF: Hotkey to Render Latency = {}ms", duration_ms);

        let mut tags = HashMap::new();
        tags.insert("type".to_string(), "ui_latency".to_string());

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        state.add_sample(PerfSample {
            name: "hotkey_to_panel".to_string(),
            duration_ms,
            tags,
            timestamp,
        });
    }
    *start_opt = None;
}

/// Measure memory usage using sysinfo (optimized per code review)
fn measure_memory() -> Result<u64, String> {
    let mut sys = System::new();
    let pid = Pid::from_u32(std::process::id());
    
    sys.refresh_process(pid);
    
    sys.process(pid)
        .map(|p| p.memory() / 1024 / 1024)
        .ok_or_else(|| "Failed to get process memory".to_string())
}

/// Benchmark PII detection with multiple scenarios (Codex enhancement)
fn benchmark_pii_detection(state: &PerfState) -> Result<(), String> {
    let scenarios = vec![
        ("short_ascii", "Call me at 13800138000.".to_string()),
        ("long_mixed", format!("联系人：张三，手机：13800138000，邮箱：test@example.com。{}", "更多内容...".repeat(1000))),
        ("unicode_heavy", format!("你好世界🌍{}", "测试文本".repeat(500))),
        (
            "nested_json",
            format!(r#"{{"users": [{{"name": "张三", "phone": "13800138000"}}, {{"name": "李四", "email": "li@test.com"}}]}}"#).repeat(100),
        ),
    ];

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    for (name, text) in scenarios {
        let start = Instant::now();
        let _ = scan_pii(&text);
        let duration_ms = start.elapsed().as_millis() as u64;

        let mut tags = HashMap::new();
        tags.insert("scenario".to_string(), name.to_string());
        tags.insert("text_length".to_string(), text.len().to_string());

        state.add_sample(PerfSample {
            name: format!("pii_detection|{}", name),
            duration_ms,
            tags,
            timestamp,
        });

        log::info!("PERF: PII Detection [{}] = {}ms", name, duration_ms);
    }

    Ok(())
}

/// Run complete performance test suite (Tauri command)
#[tauri::command]
pub async fn run_perf_suite(app: AppHandle) -> Result<PerfReport, String> {
    let state = app.state::<PerfState>();

    log::info!("PERF: Starting performance test suite");

    // 1. Measure memory usage
    let memory_mb = measure_memory()?;
    log::info!("PERF: Memory Usage = {}MB", memory_mb);

    // 2. Benchmark PII detection (multiple scenarios)
    benchmark_pii_detection(&state)?;

    // 3. Retrieve last hotkey latency
    let samples = state.samples.lock().unwrap();
    let hotkey_latency = samples
        .iter()
        .rev()
        .find(|s| s.name == "hotkey_to_panel")
        .map(|s| s.duration_ms);

    // 4. Generate report with status
    let mut metrics = vec![
        PerfMetric {
            name: "hotkey_to_panel".to_string(),
            value_ms: hotkey_latency,
            value_mb: None,
            target: 100,
            status: match hotkey_latency {
                Some(ms) if ms < 100 => "PASS".to_string(),
                Some(ms) if ms < 150 => "WARN".to_string(),
                Some(_) => "FAIL".to_string(),
                None => "NOT_MEASURED".to_string(),
            },
        },
        PerfMetric {
            name: "idle_memory".to_string(),
            value_ms: None,
            value_mb: Some(memory_mb),
            target: 50,
            status: if memory_mb < 50 {
                "PASS".to_string()
            } else if memory_mb < 55 {
                "WARN".to_string()
            } else {
                "FAIL".to_string()
            },
        },
    ];

    // Add PII metrics
    for sample in samples.iter().filter(|s| s.name.starts_with("pii_detection")) {
        metrics.push(PerfMetric {
            name: sample.name.clone(),
            value_ms: Some(sample.duration_ms),
            value_mb: None,
            target: 50,
            status: if sample.duration_ms < 50 {
                "PASS".to_string()
            } else if sample.duration_ms < 100 {
                "WARN".to_string()
            } else {
                "FAIL".to_string()
            },
        });
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let report = PerfReport {
        meta: ReportMeta {
            timestamp,
            platform: std::env::consts::OS.to_string(),
        },
        metrics,
    };

    log::info!("PERF: Test suite completed");
    Ok(report)
}
