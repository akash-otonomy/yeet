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

// Run server + cloudflared in daemon mode (called from forked child)
fn run_daemon_server(file_path: PathBuf, port: u16) {
    use std::process;

    let daemon_pid = process::id();
    let rt = Runtime::new().expect("Failed to create runtime");

    rt.block_on(async move {
        use axum::response::Response;
        use axum::body::Body;
        use axum::http::{header, StatusCode};

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

        let app = Router::new()
            .route(&serve_path, axum::routing::get(serve_file.clone()))
            .route("/", axum::routing::get(serve_file));

        let addr = format!("127.0.0.1:{}", port);
        let listener = tokio::net::TcpListener::bind(&addr).await.expect("Failed to bind");

        // Start server in background
        let server_handle = tokio::spawn(async move {
            axum::serve(listener, app).await.expect("Server failed");
        });

        // Wait a bit for server to start
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Start cloudflared
        start_cloudflared_daemon(file_path, port, daemon_pid).await;

        // Keep daemon alive
        server_handle.await.ok();
    });
}

async fn start_cloudflared_daemon(file_path: PathBuf, port: u16, daemon_pid: u32) {
    use std::io::{BufRead, BufReader};

    let mut tunnel = Command::new("cloudflared")
        .args(&["tunnel", "--url", &format!("http://localhost:{}", port)])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start cloudflared");

    if let Some(stderr) = tunnel.stderr.take() {
        let filename = file_path.file_name().unwrap().to_string_lossy().to_string();

        let reader = BufReader::new(stderr);
        let mut url_saved = false;

        for line in reader.lines() {
            if let Ok(line) = line {
                if !url_saved && line.contains("trycloudflare.com") {
                    use regex::Regex;
                    let re = Regex::new(r"https://[^\s]+\.trycloudflare\.com").unwrap();
                    if let Some(mat) = re.find(&line) {
                        let base_url = mat.as_str();
                        let url = format!("{}/{}", base_url, filename);

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
#[command(about = "🚀 Yeet files across the internet at warp speed", long_about = None)]
struct Cli {
    /// File to yeet
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
    port: u16,
    tunnel_url: Option<String>,
    frame_count: u32,
    daemon_pid: Option<u32>,
    daemon_age: Option<f64>,
}

impl App {
    fn new(file_path: PathBuf, port: u16) -> Result<Self> {
        let file_size = std::fs::metadata(&file_path)?.len();

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
            Span::styled("FILE: ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::raw(app.file_path.file_name().unwrap().to_string_lossy()),
        ]),
        Line::from(vec![
            Span::styled("SIZE: ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::raw(app.format_size()),
        ]),
        Line::from(vec![
            Span::styled("PORT: ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::raw(format!("{}", app.port)),
        ]),
    ];

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

    // Require file for normal operation
    let file = cli.file.ok_or_else(|| anyhow::anyhow!("File path required (or use --status/--kill)"))?;

    // Validate file exists
    if !file.exists() {
        anyhow::bail!("File not found: {}", file.display());
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
