use anyhow::{bail, Context, Result};
use colored::Colorize;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::process::Command;

use crate::ai::AiEngine;
use crate::config::Config;
use crate::output::{self, OutputMode};
use crate::parsers;

/// A single log event tagged with its originating service.
#[derive(Debug, Clone)]
struct ServiceEvent {
    service: String,
    timestamp: Option<String>,
    message: String,
    trace_id: Option<String>,
    severity: Option<String>,
}

/// A set of events sharing the same trace/request ID.
#[derive(Debug, Clone)]
struct CorrelatedTrace {
    trace_id: String,
    events: Vec<ServiceEvent>,
    has_errors: bool,
    root_cause_service: Option<String>,
}

pub async fn run(services: &str, since: &str, files: &[String], mode: OutputMode) -> Result<()> {
    let service_names: Vec<&str> = services.split(',').map(|s| s.trim()).collect();

    if service_names.is_empty() {
        bail!("No services specified. Use --services api,worker,db");
    }

    if mode == OutputMode::Terminal {
        output::print_info(&format!(
            "Tracing across {} services: {}",
            service_names.len(),
            services
        ));
    }

    let all_events = collect_logs(&service_names, files, since)?;

    if all_events.is_empty() {
        if mode == OutputMode::Terminal {
            output::print_info("No log data found for the specified services.");
        }
        return Ok(());
    }

    if mode == OutputMode::Terminal {
        output::print_info(&format!("Collected {} log events", all_events.len()));
    }

    let traces = correlate_events(&all_events);
    let error_traces: Vec<&CorrelatedTrace> = traces.iter().filter(|t| t.has_errors).collect();

    if mode == OutputMode::Terminal {
        println!();
        println!(
            "{} Found {} traces ({} with errors)",
            "\u{1f50d}".cyan(),
            traces.len(),
            error_traces.len()
        );
        println!();
    }

    match mode {
        OutputMode::Json | OutputMode::Ci => {
            let json = serde_json::json!({
                "total_events": all_events.len(),
                "total_traces": traces.len(),
                "error_traces": error_traces.iter().map(|t| {
                    serde_json::json!({
                        "trace_id": t.trace_id,
                        "root_cause_service": t.root_cause_service,
                        "events": t.events.iter().map(|e| {
                            serde_json::json!({
                                "service": e.service,
                                "timestamp": e.timestamp,
                                "severity": e.severity,
                                "message": e.message,
                            })
                        }).collect::<Vec<_>>()
                    })
                }).collect::<Vec<_>>()
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        OutputMode::Terminal => {
            for trace in &error_traces {
                println!("{}", "\u{2500}".repeat(60).dimmed());
                println!(
                    "{} Trace: {}",
                    "\u{1f534}",
                    trace.trace_id.yellow().bold()
                );
                if let Some(root) = &trace.root_cause_service {
                    println!(
                        "  {} {}",
                        "Root cause service:".red().bold(),
                        root.white().bold()
                    );
                }
                println!();
                for event in &trace.events {
                    let sev_icon = match event.severity.as_deref() {
                        Some("error") | Some("fatal") => "\u{274c}",
                        Some("warning") | Some("warn") => "\u{26a0}\u{fe0f}",
                        _ => "\u{2022}",
                    };
                    let ts = event.timestamp.as_deref().unwrap_or("--:--:--");
                    println!(
                        "  {} {} [{}] {}",
                        sev_icon,
                        ts.dimmed(),
                        event.service.cyan(),
                        event.message.white()
                    );
                }
                println!();
            }
        }
    }

    // Use AI to analyze error traces
    if !error_traces.is_empty() {
        let config = Config::load()?;
        if config.api_key().is_some() || config.provider == "ollama" {
            if mode == OutputMode::Terminal {
                output::print_info("Analyzing root cause with AI...");
                println!();
            }

            let prompt = build_trace_prompt(&error_traces);
            let engine = AiEngine::new(config)?;
            let response = engine.analyze(&prompt, None).await?;
            output::print_analysis(&response, mode);
        }
    }

    Ok(())
}

/// Read logs from files, docker containers, or k8s pods for each service.
fn collect_logs(
    services: &[&str],
    files: &[String],
    since: &str,
) -> Result<Vec<ServiceEvent>> {
    let mut all_events = Vec::new();
    let since_duration = normalize_since(since);
    let trace_patterns = build_trace_regex();

    for (i, service) in services.iter().enumerate() {
        let content = if i < files.len() {
            fs::read_to_string(&files[i])
                .with_context(|| format!("Failed to read log file: {}", files[i]))?
        } else {
            read_docker_logs(service, &since_duration)
                .or_else(|_| read_k8s_logs(service, &since_duration))
                .unwrap_or_default()
        };

        if content.is_empty() {
            continue;
        }

        let (_, parsed) = parsers::auto_parse(&content);

        for entry in parsed {
            let trace_id = extract_trace_id(&entry.message, &trace_patterns);
            let severity = entry.severity.map(|s| s.to_lowercase());

            all_events.push(ServiceEvent {
                service: service.to_string(),
                timestamp: entry.timestamp,
                message: entry.message,
                trace_id,
                severity,
            });
        }
    }

    Ok(all_events)
}

/// Group events by trace/request ID and identify root-cause services.
fn correlate_events(events: &[ServiceEvent]) -> Vec<CorrelatedTrace> {
    let mut trace_map: HashMap<String, Vec<ServiceEvent>> = HashMap::new();
    let mut unmatched_errors = Vec::new();

    for event in events {
        if let Some(tid) = &event.trace_id {
            trace_map.entry(tid.clone()).or_default().push(event.clone());
        } else if matches!(event.severity.as_deref(), Some("error") | Some("fatal")) {
            unmatched_errors.push(event.clone());
        }
    }

    let mut traces: Vec<CorrelatedTrace> = trace_map
        .into_iter()
        .map(|(trace_id, mut evts)| {
            evts.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

            let has_errors = evts
                .iter()
                .any(|e| matches!(e.severity.as_deref(), Some("error") | Some("fatal")));

            let root_cause_service = if has_errors {
                evts.iter()
                    .find(|e| matches!(e.severity.as_deref(), Some("error") | Some("fatal")))
                    .map(|e| e.service.clone())
            } else {
                None
            };

            CorrelatedTrace {
                trace_id,
                events: evts,
                has_errors,
                root_cause_service,
            }
        })
        .collect();

    if !unmatched_errors.is_empty() {
        let root_cause_service = unmatched_errors.first().map(|e| e.service.clone());
        traces.push(CorrelatedTrace {
            trace_id: "(no trace ID)".to_string(),
            events: unmatched_errors,
            has_errors: true,
            root_cause_service,
        });
    }

    traces.sort_by(|a, b| b.has_errors.cmp(&a.has_errors));
    traces
}

fn build_trace_regex() -> Vec<Regex> {
    [
        r"(?i)trace[_-]?id[=:\s]+([a-f0-9-]{8,})",
        r"(?i)request[_-]?id[=:\s]+([a-f0-9-]{8,})",
        r"(?i)x-request-id[=:\s]+([a-f0-9-]{8,})",
        r"(?i)correlation[_-]?id[=:\s]+([a-f0-9-]{8,})",
        r#""trace_id"\s*:\s*"([^"]+)""#,
        r#""request_id"\s*:\s*"([^"]+)""#,
        r#""traceId"\s*:\s*"([^"]+)""#,
        r#""requestId"\s*:\s*"([^"]+)""#,
    ]
    .iter()
    .filter_map(|p| Regex::new(p).ok())
    .collect()
}

fn extract_trace_id(message: &str, patterns: &[Regex]) -> Option<String> {
    for re in patterns {
        if let Some(caps) = re.captures(message) {
            if let Some(m) = caps.get(1) {
                return Some(m.as_str().to_string());
            }
        }
    }
    None
}

fn normalize_since(since: &str) -> String {
    since.trim().trim_end_matches("ago").trim().to_string()
}

fn read_docker_logs(container: &str, since: &str) -> Result<String> {
    let output = Command::new("docker")
        .args(["logs", "--since", since, container])
        .output()
        .context("docker command not found")?;

    if !output.status.success() {
        bail!(
            "docker logs failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let mut logs = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr_str = String::from_utf8_lossy(&output.stderr);
    if !stderr_str.is_empty() {
        logs.push('\n');
        logs.push_str(&stderr_str);
    }
    Ok(logs)
}

fn read_k8s_logs(pod: &str, since: &str) -> Result<String> {
    let output = Command::new("kubectl")
        .args(["logs", &format!("--since={since}"), pod])
        .output()
        .context("kubectl command not found")?;

    if !output.status.success() {
        bail!(
            "kubectl logs failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn build_trace_prompt(error_traces: &[&CorrelatedTrace]) -> String {
    let mut prompt = String::from("Multi-service error correlation analysis:\n\n");

    for trace in error_traces.iter().take(10) {
        prompt.push_str(&format!("Trace ID: {}\n", trace.trace_id));
        if let Some(root) = &trace.root_cause_service {
            prompt.push_str(&format!("Suspected root cause service: {root}\n"));
        }
        prompt.push_str("Events:\n");
        for event in &trace.events {
            let ts = event.timestamp.as_deref().unwrap_or("unknown");
            let sev = event.severity.as_deref().unwrap_or("info");
            prompt.push_str(&format!(
                "  [{ts}] [{sev}] [{}] {}\n",
                event.service, event.message
            ));
        }
        prompt.push('\n');
    }

    prompt.push_str(
        "Analyze the error traces above. Identify the root cause service and explain the failure chain across services.",
    );
    prompt
}
