// Dioxus web UI components (not currently integrated into main server)
// Ready for future use when we want to replace the Alpine.js admin dashboard

use crate::shared::{RequestLog, ServerStats};
use dioxus::prelude::*;

/// Retro color palette (matching TUI theme)
#[allow(dead_code)]
const CYAN: &str = "#00FFFF";
#[allow(dead_code)]
const MAGENTA: &str = "#FF00FF";
#[allow(dead_code)]
const YELLOW: &str = "#FFFF00";
#[allow(dead_code)]
const GREEN: &str = "#00FF9F";
#[allow(dead_code)]
const ORANGE: &str = "#FF8000";
#[allow(dead_code)]
const DARK_BG: &str = "#14141E";
#[allow(dead_code)]
const GRAY: &str = "#808080";
#[allow(dead_code)]
const LIGHT_GRAY: &str = "#C0C0C0";

#[component]
pub fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./assets/style.css") }

        div {
            class: "container",
            style: "background: {DARK_BG}; color: {LIGHT_GRAY}; min-height: 100vh; font-family: 'Roboto Mono', monospace; padding: 20px;",

            // Colorful YEET logo
            Logo {}

            // Stats section
            Stats {}

            // Live requests
            LiveLog {}
        }
    }
}

#[component]
fn Logo() -> Element {
    rsx! {
        div {
            class: "logo",
            style: "text-align: center; margin: 40px 0; font-weight: bold; font-size: 24px;",

            div { style: "color: {MAGENTA};", "██╗   ██╗" }
            span { style: "color: {CYAN};", "███████╗" }
            span { style: "color: {YELLOW};", "███████╗" }
            span { style: "color: {GREEN};", "████████╗" }

            div {
                style: "margin-top: 10px; font-size: 14px; color: {GRAY};",
                "🚀 Yeet files across the internet at warp speed"
            }
        }
    }
}

#[component]
fn Stats() -> Element {
    // TODO: Fetch real stats from /api/stats endpoint
    let _stats = use_signal(|| ServerStats {
        uptime_secs: 7515,
        total_requests: 1247,
        total_bytes_sent: 163_680_051,
        current_speed_bps: 2_200_000,
        active_connections: 8,
        unique_ips: 42,
        requests_per_minute: 24,
    });

    rsx! {
        div {
            class: "stats-grid",
            style: "
                display: grid;
                grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
                gap: 20px;
                margin: 40px 0;
            ",

            StatCard {
                label: "UPTIME",
                value: "2h 15m",
                color: CYAN,
            }

            StatCard {
                label: "REQUESTS",
                value: "1,247",
                color: MAGENTA,
            }

            StatCard {
                label: "BANDWIDTH",
                value: "156.2 MB",
                color: YELLOW,
            }

            StatCard {
                label: "SPEED",
                value: "2.1 MB/s",
                color: GREEN,
            }
        }

        // Progress bar for requests/min
        div {
            style: "margin: 20px 0;",

            div {
                style: "color: {LIGHT_GRAY}; margin-bottom: 10px;",
                "ACTIVITY: 24 requests/min"
            }

            ProgressBar {
                percent: 0.75,
                color: ORANGE,
            }
        }
    }
}

#[component]
fn StatCard(label: String, value: String, color: String) -> Element {
    rsx! {
        div {
            class: "stat-card",
            style: "
                border: 2px solid {color};
                border-radius: 8px;
                padding: 20px;
                background: rgba(255, 255, 255, 0.05);
                transition: all 0.3s;
            ",

            div {
                style: "color: {color}; font-size: 12px; margin-bottom: 10px;",
                "{label}"
            }

            div {
                style: "color: {LIGHT_GRAY}; font-size: 32px; font-weight: bold;",
                "{value}"
            }
        }
    }
}

#[component]
fn ProgressBar(percent: f64, color: String) -> Element {
    let width = (percent * 100.0).min(100.0);

    rsx! {
        div {
            style: "
                width: 100%;
                height: 30px;
                background: rgba(255, 255, 255, 0.1);
                border-radius: 15px;
                overflow: hidden;
                position: relative;
            ",

            div {
                style: "
                    width: {width}%;
                    height: 100%;
                    background: linear-gradient(90deg, {MAGENTA} 0%, {CYAN} 100%);
                    transition: width 0.3s;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    color: white;
                    font-weight: bold;
                ",
                "{width:.0}%"
            }
        }
    }
}

#[component]
fn LiveLog() -> Element {
    // TODO: Fetch real logs from /api/logs endpoint
    let _logs = use_signal(|| {
        vec![
            RequestLog {
                timestamp: 1234567890,
                method: "GET".to_string(),
                path: "/cat.jpg".to_string(),
                status: 200,
                size_bytes: 2_400_000,
                user_agent: "Chrome".to_string(),
                ip: "192.168.1.5".to_string(),
            },
            RequestLog {
                timestamp: 1234567888,
                method: "GET".to_string(),
                path: "/cat.jpg".to_string(),
                status: 200,
                size_bytes: 2_400_000,
                user_agent: "Safari".to_string(),
                ip: "10.0.1.23".to_string(),
            },
        ]
    });

    rsx! {
        div {
            class: "live-log",
            style: "margin: 40px 0;",

            h2 {
                style: "color: {CYAN}; margin-bottom: 20px;",
                "📡 LIVE REQUESTS"
            }

            div {
                style: "
                    border: 2px solid {GRAY};
                    border-radius: 8px;
                    padding: 20px;
                    background: rgba(255, 255, 255, 0.03);
                    font-family: 'Roboto Mono', monospace;
                ",

                for log in _logs() {
                    LogEntry { log: log.clone() }
                }
            }
        }
    }
}

#[component]
fn LogEntry(log: RequestLog) -> Element {
    let status_color = match log.status {
        200..=299 => GREEN,
        300..=399 => YELLOW,
        _ => ORANGE,
    };

    rsx! {
        div {
            style: "
                display: grid;
                grid-template-columns: 80px 60px 200px 60px 80px 100px 150px;
                gap: 10px;
                padding: 10px 0;
                border-bottom: 1px solid rgba(255, 255, 255, 0.1);
                font-size: 13px;
            ",

            span { style: "color: {GRAY};", "00:23:45" }
            span { style: "color: {CYAN};", "{log.method}" }
            span { style: "color: {LIGHT_GRAY};", "{log.path}" }
            span { style: "color: {status_color};", "{log.status}" }
            span { style: "color: {YELLOW};", "2.4MB" }
            span { style: "color: {MAGENTA};", "{log.user_agent}" }
            span { style: "color: {GRAY};", "{log.ip}" }
        }
    }
}
