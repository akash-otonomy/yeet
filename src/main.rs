use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, Paragraph, Wrap},
    Frame, Terminal,
};
use std::{
    io,
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
    fs,
};
use tokio::runtime::Runtime;
use axum::Router;
use serde::{Serialize, Deserialize};
use nix::unistd::{fork, ForkResult, setsid};

const YEET_LOGO: &str = r#"
█  █ ███ ███ ███   ███ █ █
 ██  █   █    █  █ █   █▀█
  █  ███ ███  █  █ ███ █ █
>> yeet stuff across the internet
"#;

// Spawn a daemon process that runs server + cloudflared
fn spawn_daemon(file_path: PathBuf, port: u16) -> Result<u32> {
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            // Parent process - return daemon PID
            let pid = child.as_raw() as u32;
            Ok(pid)
        }
        Ok(ForkResult::Child) => {
            // Child process - become daemon

            // Create new session (detach from terminal)
            setsid().expect("Failed to create new session");

            // Redirect stdin/stdout/stderr to /dev/null
            use std::fs::OpenOptions;
            let dev_null = OpenOptions::new()
                .read(true)
                .write(true)
                .open("/dev/null")
                .expect("Failed to open /dev/null");

            use std::os::unix::io::AsRawFd;
            use nix::unistd::dup2;
            let null_fd = dev_null.as_raw_fd();
            dup2(null_fd, 0).ok(); // stdin
            dup2(null_fd, 1).ok(); // stdout
            dup2(null_fd, 2).ok(); // stderr

            // Run server and cloudflared
            run_daemon_server(file_path, port);

            // Should never reach here
            std::process::exit(0);
        }
        Err(e) => {
            anyhow::bail!("Failed to fork: {}", e);
        }
    }
}

// Helper to format bytes into human-readable size
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

// Run server + cloudflared in daemon mode (called from forked child)
fn run_daemon_server(file_path: PathBuf, port: u16) {
    use std::process;

    let daemon_pid = process::id();
    let rt = Runtime::new().expect("Failed to create runtime");

    rt.block_on(async move {
        use axum::response::Response;
        use axum::body::Body;
        use axum::http::{header, StatusCode};
        use tower_http::services::ServeDir;

        let is_dir = file_path.is_dir();

        let app = if is_dir {
            // Serve directory with sick retro UI
            let dir_path = file_path.clone();
            let index_handler = move |req: axum::extract::Request| {
                let base_path = dir_path.clone();
                async move {
                    use std::path::Path;

                    // Get path from URI
                    let req_path = req.uri().path();

                    // Construct full path
                    let full_path = if req_path.is_empty() || req_path == "/" {
                        base_path.clone()
                    } else {
                        base_path.join(req_path.trim_start_matches('/'))
                    };

                    // If it's a file, serve it
                    if full_path.is_file() {
                        match tokio::fs::read(&full_path).await {
                            Ok(contents) => {
                                return Response::builder()
                                    .status(StatusCode::OK)
                                    .header(header::CONTENT_TYPE, "application/octet-stream")
                                    .header(header::CONTENT_DISPOSITION,
                                        format!("attachment; filename=\"{}\"",
                                            full_path.file_name().unwrap().to_string_lossy()))
                                    .body(Body::from(contents))
                                    .unwrap();
                            }
                            Err(_) => {
                                return Response::builder()
                                    .status(StatusCode::NOT_FOUND)
                                    .body(Body::from("File not found"))
                                    .unwrap();
                            }
                        }
                    }

                    // If not a directory, 404
                    if !full_path.is_dir() {
                        return Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .body(Body::from("Not found"))
                            .unwrap();
                    }

                    let path = full_path;
                    let dir_name = path.file_name().unwrap().to_string_lossy().to_string();
                    let current_path = req_path.to_string();
                    let mut files = Vec::new();

                    if let Ok(entries) = std::fs::read_dir(&path) {
                        for entry in entries.flatten() {
                            if let (Ok(name), Ok(metadata)) = (entry.file_name().into_string(), entry.metadata()) {
                                let size = metadata.len();
                                let is_file = metadata.is_file();
                                files.push((name, size, is_file));
                            }
                        }
                    }

                    files.sort_by(|a, b| a.0.cmp(&b.0));

                    let mut file_list = String::new();
                    for (name, size, is_file) in files {
                        let icon = if is_file { "📄" } else { "📁" };
                        let size_str = if is_file {
                            format_bytes(size)
                        } else {
                            "-".to_string()
                        };
                        // Build absolute path for links
                        let link_path = if current_path == "/" || current_path.is_empty() {
                            format!("/{}", name)
                        } else {
                            format!("{}/{}", current_path.trim_end_matches('/'), name)
                        };
                        file_list.push_str(&format!(
                            r#"{{ name: '{}', path: '{}', size: '{}', sizeBytes: {}, icon: '{}', isFile: {} }},"#,
                            name.replace("'", "\\'"), link_path.replace("'", "\\'"), size_str, size, icon, is_file
                        ));
                    }

                    let html = format!(r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>YEET // {}</title>
    <link href="https://fonts.googleapis.com/css2?family=Roboto+Mono:wght@400;700&display=swap" rel="stylesheet">
    <script defer src="https://cdn.jsdelivr.net/npm/alpinejs@3.x.x/dist/cdn.min.js"></script>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{
            background: #0a0e27;
            color: #00ff9f;
            font-family: 'Roboto Mono', monospace;
            padding: 2rem;
            min-height: 100vh;
        }}
        .container {{ max-width: 1200px; margin: 0 auto; }}
        .header {{
            border: 2px solid #00ff9f;
            padding: 1.5rem;
            margin-bottom: 2rem;
            background: rgba(0, 255, 159, 0.05);
        }}
        .logo {{
            font-size: 2rem;
            font-weight: bold;
            color: #00d4ff;
            text-shadow: 0 0 10px #00d4ff;
            margin-bottom: 0.5rem;
        }}
        .subtitle {{
            color: #ff00ff;
            font-size: 0.9rem;
        }}
        .controls {{
            display: flex;
            gap: 1rem;
            margin-bottom: 1rem;
            flex-wrap: wrap;
        }}
        input, select {{
            background: #1a1f3a;
            border: 1px solid #00ff9f;
            color: #00ff9f;
            padding: 0.5rem 1rem;
            font-family: 'Roboto Mono', monospace;
            font-size: 0.9rem;
        }}
        input:focus, select:focus {{
            outline: none;
            box-shadow: 0 0 10px rgba(0, 255, 159, 0.5);
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            border: 2px solid #00ff9f;
        }}
        th {{
            background: rgba(0, 255, 159, 0.1);
            padding: 1rem;
            text-align: left;
            border-bottom: 2px solid #00ff9f;
            cursor: pointer;
            user-select: none;
            color: #00d4ff;
        }}
        th:hover {{
            background: rgba(0, 255, 159, 0.2);
        }}
        td {{
            padding: 0.75rem 1rem;
            border-bottom: 1px solid rgba(0, 255, 159, 0.2);
        }}
        tr:hover {{
            background: rgba(0, 255, 159, 0.05);
        }}
        a {{
            color: #00ff9f;
            text-decoration: none;
            display: flex;
            align-items: center;
            gap: 0.5rem;
        }}
        a:hover {{
            color: #00d4ff;
            text-shadow: 0 0 5px #00d4ff;
        }}
        .icon {{ font-size: 1.2rem; }}
        .stats {{
            margin-top: 1rem;
            padding: 1rem;
            border: 1px solid rgba(255, 0, 255, 0.3);
            background: rgba(255, 0, 255, 0.05);
            color: #ff00ff;
            font-size: 0.85rem;
        }}
        .sort-indicator {{
            font-size: 0.7rem;
            margin-left: 0.5rem;
            color: #ff00ff;
        }}
    </style>
</head>
<body x-data="fileManager()">
    <div class="container">
        <div class="header">
            <div class="logo">█ YEET.SH █</div>
            <div class="subtitle">// {}</div>
        </div>

        <div class="controls">
            <input
                type="text"
                x-model="search"
                placeholder="⚡ SEARCH FILES..."
                style="flex: 1; min-width: 200px;">
            <select x-model="filter">
                <option value="all">ALL FILES</option>
                <option value="files">FILES ONLY</option>
                <option value="dirs">DIRS ONLY</option>
            </select>
        </div>

        <table>
            <thead>
                <tr>
                    <th @click="sortBy('name')" style="width: 50%">
                        NAME
                        <span class="sort-indicator" x-show="sortKey === 'name'" x-text="sortAsc ? '▲' : '▼'"></span>
                    </th>
                    <th @click="sortBy('size')" style="width: 30%">
                        SIZE
                        <span class="sort-indicator" x-show="sortKey === 'size'" x-text="sortAsc ? '▲' : '▼'"></span>
                    </th>
                    <th style="width: 20%">TYPE</th>
                </tr>
            </thead>
            <tbody>
                <template x-for="file in filteredFiles" :key="file.name">
                    <tr>
                        <td>
                            <a :href="file.path">
                                <span class="icon" x-text="file.icon"></span>
                                <span x-text="file.name"></span>
                            </a>
                        </td>
                        <td x-text="file.size"></td>
                        <td x-text="file.isFile ? 'FILE' : 'DIR'"></td>
                    </tr>
                </template>
            </tbody>
        </table>

        <div class="stats">
            <span x-text="stats"></span>
        </div>
    </div>

    <script>
        function fileManager() {{
            return {{
                files: [{}],
                search: '',
                filter: 'all',
                sortKey: 'name',
                sortAsc: true,

                sortBy(key) {{
                    if (this.sortKey === key) {{
                        this.sortAsc = !this.sortAsc;
                    }} else {{
                        this.sortKey = key;
                        this.sortAsc = true;
                    }}
                }},

                get filteredFiles() {{
                    let filtered = this.files.filter(f => {{
                        if (this.search && !f.name.toLowerCase().includes(this.search.toLowerCase())) return false;
                        if (this.filter === 'files' && !f.isFile) return false;
                        if (this.filter === 'dirs' && f.isFile) return false;
                        return true;
                    }});

                    filtered.sort((a, b) => {{
                        let aVal = this.sortKey === 'size' ? a.sizeBytes : a.name.toLowerCase();
                        let bVal = this.sortKey === 'size' ? b.sizeBytes : b.name.toLowerCase();
                        return this.sortAsc ?
                            (aVal < bVal ? -1 : 1) :
                            (aVal > bVal ? -1 : 1);
                    }});

                    return filtered;
                }},

                get stats() {{
                    let total = this.filteredFiles.length;
                    let files = this.filteredFiles.filter(f => f.isFile).length;
                    let dirs = total - files;
                    return `▸ SHOWING ${{total}} ITEMS // ${{files}} FILES // ${{dirs}} DIRECTORIES`;
                }}
            }}
        }}
    </script>
</body>
</html>"#, dir_name, dir_name, file_list);

                    Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                        .body(Body::from(html))
                        .unwrap()
                }
            };

            Router::new()
                .fallback(index_handler)
        } else {
            // Serve single file
            let filename = file_path.file_name().unwrap().to_string_lossy().to_string();
            let serve_path = format!("/{}", filename);
            let file_path_clone = file_path.clone();

            // Handler that serves the file
            let serve_file = move || {
                let path = file_path_clone.clone();
                async move {
                    match tokio::fs::read(&path).await {
                        Ok(contents) => {
                            Response::builder()
                                .status(StatusCode::OK)
                                .header(header::CONTENT_TYPE, "application/octet-stream")
                                .header(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"",
                                    path.file_name().unwrap().to_string_lossy()))
                                .body(Body::from(contents))
                                .unwrap()
                        }
                        Err(_) => {
                            Response::builder()
                                .status(StatusCode::NOT_FOUND)
                                .body(Body::from("File not found"))
                                .unwrap()
                        }
                    }
                }
            };

            Router::new()
                .route(&serve_path, axum::routing::get(serve_file.clone()))
                .route("/", axum::routing::get(serve_file))
        };

        let addr = format!("127.0.0.1:{}", port);
        let listener = tokio::net::TcpListener::bind(&addr).await.expect("Failed to bind");

        // Start server in background
        let server_handle = tokio::spawn(async move {
            axum::serve(listener, app).await.expect("Server failed");
        });

        // Wait a bit for server to start
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Start cloudflared
        start_cloudflared_daemon(file_path, port, daemon_pid, is_dir).await;

        // Keep daemon alive
        server_handle.await.ok();
    });
}

async fn start_cloudflared_daemon(file_path: PathBuf, port: u16, daemon_pid: u32, is_dir: bool) {
    use std::io::{BufRead, BufReader};

    let mut tunnel = Command::new("cloudflared")
        .args(&["tunnel", "--url", &format!("http://localhost:{}", port)])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start cloudflared");

    if let Some(stderr) = tunnel.stderr.take() {
        let reader = BufReader::new(stderr);
        let mut url_saved = false;

        for line in reader.lines() {
            if let Ok(line) = line {
                if !url_saved && line.contains("trycloudflare.com") {
                    use regex::Regex;
                    let re = Regex::new(r"https://[^\s]+\.trycloudflare\.com").unwrap();
                    if let Some(mat) = re.find(&line) {
                        let base_url = mat.as_str();

                        // For directories, use base URL; for files, append filename
                        let url = if is_dir {
                            base_url.to_string()
                        } else {
                            let filename = file_path.file_name().unwrap().to_string_lossy().to_string();
                            format!("{}/{}", base_url, filename)
                        };

                        // Save state
                        use std::time::{SystemTime, UNIX_EPOCH};
                        let state = TunnelState {
                            url,
                            pid: daemon_pid,
                            port,
                            file_path: file_path.to_string_lossy().to_string(),
                            created_at: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                        };
                        let _ = state.save();
                        url_saved = true;
                        // Continue reading stderr to keep pipe open
                    }
                }
            }
        }
    }

    // Keep tunnel alive forever
    let _ = tunnel.wait();
}

#[derive(Serialize, Deserialize, Clone)]
struct TunnelState {
    url: String,
    pid: u32,
    port: u16,
    file_path: String,
    created_at: u64, // unix timestamp
}

impl TunnelState {
    fn state_file() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let yeet_dir = PathBuf::from(home).join(".yeet");
        fs::create_dir_all(&yeet_dir).ok();
        yeet_dir.join("tunnel.state")
    }

    fn load() -> Option<Self> {
        let path = Self::state_file();
        if path.exists() {
            let content = fs::read_to_string(path).ok()?;
            serde_json::from_str(&content).ok()
        } else {
            None
        }
    }

    fn save(&self) -> Result<()> {
        let path = Self::state_file();
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    fn delete() {
        let path = Self::state_file();
        let _ = fs::remove_file(path);
    }

    fn is_tunnel_alive(&self) -> bool {
        // Check if the process is still running
        #[cfg(unix)]
        {
            use std::process::Command as StdCommand;
            StdCommand::new("kill")
                .arg("-0")
                .arg(self.pid.to_string())
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }
        #[cfg(not(unix))]
        {
            // Windows fallback - always assume dead
            false
        }
    }

    fn age_hours(&self) -> f64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        (now - self.created_at) as f64 / 3600.0
    }
}

#[derive(Parser)]
#[command(name = "yeet")]
#[command(about = "🚀 Yeet files and directories across the internet at warp speed", long_about = None)]
struct Cli {
    /// File or directory to yeet
    file: Option<PathBuf>,

    /// Port for HTTP server (default: 8000)
    #[arg(short, long, default_value = "8000")]
    port: u16,

    /// Keep tunnel alive in background (daemon mode)
    #[arg(short, long)]
    daemon: bool,

    /// Show existing tunnel status
    #[arg(long)]
    status: bool,

    /// Kill existing tunnel daemon
    #[arg(long)]
    kill: bool,
}

#[derive(Clone)]
struct DownloadMetrics {
    downloads: Arc<AtomicU64>,
    bytes_sent: Arc<AtomicU64>,
    last_download_time: Arc<Mutex<Option<Instant>>>,
    active_downloads: Arc<AtomicU64>,
}

impl DownloadMetrics {
    fn new() -> Self {
        Self {
            downloads: Arc::new(AtomicU64::new(0)),
            bytes_sent: Arc::new(AtomicU64::new(0)),
            last_download_time: Arc::new(Mutex::new(None)),
            active_downloads: Arc::new(AtomicU64::new(0)),
        }
    }

    fn start_download(&self) {
        self.downloads.fetch_add(1, Ordering::Relaxed);
        self.active_downloads.fetch_add(1, Ordering::Relaxed);
        *self.last_download_time.lock().unwrap() = Some(Instant::now());
    }

    fn finish_download(&self) {
        self.active_downloads.fetch_sub(1, Ordering::Relaxed);
    }

    fn add_bytes(&self, bytes: u64) {
        self.bytes_sent.fetch_add(bytes, Ordering::Relaxed);
    }

    fn get_stats(&self) -> (u64, u64, u64, Option<f64>) {
        let downloads = self.downloads.load(Ordering::Relaxed);
        let bytes = self.bytes_sent.load(Ordering::Relaxed);
        let active = self.active_downloads.load(Ordering::Relaxed);

        let speed = if let Some(start) = *self.last_download_time.lock().unwrap() {
            let elapsed = start.elapsed().as_secs_f64();
            if elapsed > 0.1 && bytes > 0 {
                // Calculate average speed across all downloads
                Some(bytes as f64 / elapsed / 1024.0 / 1024.0) // MB/s
            } else {
                None
            }
        } else {
            None
        };

        (downloads, bytes, active, speed)
    }
}

struct App {
    file_path: PathBuf,
    file_size: u64,
    is_dir: bool,
    port: u16,
    tunnel_url: Option<String>,
    frame_count: u32,
    daemon_pid: Option<u32>,
    daemon_age: Option<f64>,
}

impl App {
    fn new(file_path: PathBuf, port: u16) -> Result<Self> {
        let metadata = std::fs::metadata(&file_path)?;
        let is_dir = metadata.is_dir();
        let file_size = if is_dir { 0 } else { metadata.len() };

        // Load initial state from daemon
        let (tunnel_url, daemon_pid, daemon_age) = if let Some(state) = TunnelState::load() {
            let age = state.age_hours();
            (Some(state.url.clone()), Some(state.pid), Some(age))
        } else {
            (None, None, None)
        };

        Ok(Self {
            file_path,
            file_size,
            is_dir,
            port,
            tunnel_url,
            frame_count: 0,
            daemon_pid,
            daemon_age,
        })
    }

    fn refresh_state(&mut self) {
        // Reload state from daemon
        if let Some(state) = TunnelState::load() {
            let age = state.age_hours();
            self.tunnel_url = Some(state.url.clone());
            self.daemon_pid = Some(state.pid);
            self.daemon_age = Some(age);
        }
    }

    fn tick(&mut self) {
        self.frame_count = self.frame_count.wrapping_add(1);
        // Refresh state from daemon every few ticks
        if self.frame_count % 30 == 0 {
            self.refresh_state();
        }
    }

    fn format_size(&self) -> String {
        let size = self.file_size as f64;
        if size < 1024.0 {
            format!("{:.0} B", size)
        } else if size < 1024.0 * 1024.0 {
            format!("{:.1} KB", size / 1024.0)
        } else if size < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.1} MB", size / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", size / (1024.0 * 1024.0 * 1024.0))
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let size = f.area();

    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),  // Logo
            Constraint::Length(5),  // Info
            Constraint::Min(6),     // URL panel
            Constraint::Length(1),  // Footer
        ])
        .split(size);

    // Render logo
    let logo = Paragraph::new(YEET_LOGO)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Left);
    f.render_widget(logo, chunks[0]);

    // Info box
    let mut info_lines = vec![
        Line::from(vec![
            Span::styled(if app.is_dir { "DIR: " } else { "FILE: " },
                Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::raw(app.file_path.file_name().unwrap().to_string_lossy()),
        ]),
    ];

    if !app.is_dir {
        info_lines.push(Line::from(vec![
            Span::styled("SIZE: ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::raw(app.format_size()),
        ]));
    }

    info_lines.push(Line::from(vec![
        Span::styled("PORT: ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        Span::raw(format!("{}", app.port)),
    ]));

    if let Some(pid) = app.daemon_pid {
        info_lines.push(Line::from(vec![
            Span::styled("DAEMON PID: ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{}", pid), Style::default().fg(Color::Green)),
        ]));
    }

    let info = Paragraph::new(info_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .border_type(BorderType::Thick)
                .title("▓▒INFO▒▓")
                .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(info, chunks[1]);

    // URL panel
    if let Some(url) = &app.tunnel_url {
        let mut url_lines = vec![
            Line::from(vec![
                Span::styled(">> ", Style::default().fg(Color::Green)),
                Span::styled(url, Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED)),
            ]),
            Line::from(""),
        ];

        if let Some(age) = app.daemon_age {
            url_lines.push(Line::from(vec![
                Span::styled("UPTIME: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(format!("{:.1} hours", age)),
            ]));
            url_lines.push(Line::from(""));
        }

        url_lines.push(Line::from(vec![
            Span::styled("░▒▓ ", Style::default().fg(Color::Magenta)),
            Span::raw("Daemon running in background"),
        ]));
        url_lines.push(Line::from(vec![
            Span::styled("░▒▓ ", Style::default().fg(Color::Magenta)),
            Span::raw("Press [q] to exit TUI (tunnel stays alive)"),
        ]));
        url_lines.push(Line::from(vec![
            Span::styled("░▒▓ ", Style::default().fg(Color::Magenta)),
            Span::raw("Use 'yeet --kill' to stop daemon"),
        ]));

        let url_panel = Paragraph::new(url_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green))
                    .border_type(BorderType::Thick)
                    .title("▓▒░YEETED░▒▓")
                    .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            )
            .alignment(Alignment::Left);
        f.render_widget(url_panel, chunks[2]);
    }

    // Footer
    let footer = Paragraph::new("[q]uit TUI (daemon stays alive)")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Left);
    f.render_widget(footer, chunks[3]);
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        app.tick();

        // Handle input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        return Ok(());
                    }
                    KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }
    }
}

fn main() -> Result<()> {
    let _ = color_eyre::install();

    let cli = Cli::parse();

    // Handle --status flag
    if cli.status {
        if let Some(state) = TunnelState::load() {
            if state.is_tunnel_alive() {
                println!("✓ Tunnel is ALIVE");
                println!("  URL:     {}", state.url);
                println!("  File:    {}", state.file_path);
                println!("  Port:    {}", state.port);
                println!("  PID:     {}", state.pid);
                println!("  Age:     {:.1} hours", state.age_hours());
            } else {
                println!("✗ Tunnel is DEAD (process not running)");
                println!("  Last URL: {}", state.url);
                TunnelState::delete();
            }
        } else {
            println!("No tunnel state found");
        }
        return Ok(());
    }

    // Handle --kill flag
    if cli.kill {
        if let Some(state) = TunnelState::load() {
            if state.is_tunnel_alive() {
                #[cfg(unix)]
                {
                    // Kill the entire process group to stop daemon + cloudflared
                    std::process::Command::new("kill")
                        .arg("-9")
                        .arg(format!("-{}", state.pid))
                        .output()?;
                    println!("✓ Killed tunnel daemon and children (PID {})", state.pid);
                }
                #[cfg(not(unix))]
                {
                    println!("✗ Kill not supported on this platform");
                }
            } else {
                println!("✗ Tunnel already dead");
            }
            TunnelState::delete();
        } else {
            println!("No tunnel to kill");
        }
        return Ok(());
    }

    // Require file/directory for normal operation
    let file = cli.file.ok_or_else(|| anyhow::anyhow!("File or directory path required (or use --status/--kill)"))?;

    // Validate path exists
    if !file.exists() {
        anyhow::bail!("Path not found: {}", file.display());
    }

    // Check for existing tunnel
    let daemon_exists = if let Some(state) = TunnelState::load() {
        if state.is_tunnel_alive() && state.port == cli.port {
            println!("Found existing tunnel (age: {:.1}h)", state.age_hours());
            println!("URL: {}", state.url);
            println!("\nReusing tunnel... Starting TUI...");
            println!("Press 'q' to exit TUI (tunnel stays alive)");
            println!("Use 'yeet --kill' to stop the daemon");
            thread::sleep(Duration::from_secs(2));
            true
        } else {
            TunnelState::delete();
            false
        }
    } else {
        false
    };

    // Spawn daemon if needed
    if !daemon_exists {
        println!("🚀 Spawning daemon...");
        let daemon_pid = spawn_daemon(file.clone(), cli.port)?;
        println!("✓ Daemon started (PID: {})", daemon_pid);
        println!("⏳ Waiting for tunnel URL...");

        // Wait for state file to be created (max 30 seconds)
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(30) {
            if let Some(state) = TunnelState::load() {
                println!("✓ Tunnel ready!");
                println!("  URL: {}", state.url);
                break;
            }
            thread::sleep(Duration::from_millis(500));
        }

        if TunnelState::load().is_none() {
            anyhow::bail!("Daemon failed to create tunnel within 30 seconds");
        }
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run (TUI only, daemon runs independently)
    let app = App::new(file, cli.port)?;
    let res = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }

    Ok(())
}
