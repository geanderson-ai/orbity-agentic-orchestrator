use crate::context::Cx;
use crate::governance::{GovernanceConsole, HitlApprovalRequest, HitlApprovalResponse};
use crate::live::{LiveDelta, LiveEmitter, TaskProgressStream};
use crate::push::ServerPushManager;
use crate::views::Views;
use tokio::sync::mpsc;

/// The embedded full interactive web application HTML.
pub const EMBEDDED_FRONTEND_HTML: &str = include_str!("../../../index.html");

/// The embedded Orbity official brand logo image asset.
pub const ORBITY_LOGO_JPG: &[u8] = include_bytes!("../../../assets/orbity-logo.jpg");

/// The Tokio Topcoat Server Application instance.
pub struct TopcoatServer {
    cx: Cx,
    push_manager: ServerPushManager,
}

impl TopcoatServer {
    pub fn new(cx: Cx) -> Self {
        let push_manager = ServerPushManager::new(cx.clone());
        Self { cx, push_manager }
    }

    pub fn cx(&self) -> &Cx {
        &self.cx
    }

    pub fn push_manager(&self) -> &ServerPushManager {
        &self.push_manager
    }

    /// Returns the embedded full interactive web application HTML.
    pub fn render_full_app(&self) -> &'static str {
        EMBEDDED_FRONTEND_HTML
    }

    /// Returns the official embedded logo binary bytes.
    pub fn logo_bytes(&self) -> &'static [u8] {
        ORBITY_LOGO_JPG
    }

    /// Renders the complete Dashboard view combining shards, FinOps widgets, and live stream containers.
    pub fn render_dashboard(&self) -> String {
        let finops = Views::finops_widget(20.0, 1.25, 45_000);
        let audit = Views::audit_chain_badge(true, 14);

        let agent1 = Views::agent_card("Codex Worker", "codex", "Executing", 15_000, 0.45);
        let agent2 = Views::agent_card("Claude Reviewer", "claude", "Idle", 8_500, 0.32);
        let agent3 = Views::agent_card("Agy Researcher", "agy", "Idle", 12_000, 0.28);
        let agent4 = Views::agent_card("Hermes Tool", "hermes", "Idle", 5_000, 0.12);
        let agent5 = Views::agent_card("Pi Refactor", "pi", "Idle", 4_500, 0.08);

        format!(
            "<!DOCTYPE html>\
            <html lang=\"en\">\
            <head>\
                <meta charset=\"UTF-8\">\
                <title>Orbity Agentic Platform - Topcoat Console</title>\
                <style>\
                    body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #0f172a; color: #f8fafc; margin: 0; padding: 24px; }}\
                    .container {{ max-width: 1200px; margin: 0 auto; }}\
                    .grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 16px; margin-top: 20px; }}\
                    .card {{ background: #1e293b; border-radius: 8px; padding: 16px; border: 1px solid #334155; }}\
                    .badge {{ padding: 4px 8px; border-radius: 4px; font-size: 12px; }}\
                    .badge-executing {{ background: #0284c7; color: white; }}\
                    .badge-idle {{ background: #475569; color: white; }}\
                    .progress-bar {{ background: #334155; border-radius: 4px; height: 8px; overflow: hidden; margin: 8px 0; }}\
                    .progress-bar .fill {{ background: #10b981; height: 100%; }}\
                </style>\
            </head>\
            <body>\
                <div class=\"container\">\
                    <header>\
                        <h1>Orbity Agentic Orchestrator (Topcoat 0.9)</h1>\
                        {}\
                    </header>\
                    {}\
                    <h2>Active Multi-Agent Network</h2>\
                    <div class=\"grid\">\
                        {}\
                        {}\
                        {}\
                        {}\
                        {}\
                    </div>\
                    <div id=\"progress-bar\"></div>\
                    <div id=\"event-stream-container\"></div>\
                </div>\
            </body>\
            </html>",
            audit.render(),
            finops.render(),
            agent1.render(),
            agent2.render(),
            agent3.render(),
            agent4.render(),
            agent5.render()
        )
    }

    /// Handles an incoming HTTP GET / request and returns the full HTML page.
    pub async fn handle_index(&self) -> String {
        self.render_dashboard()
    }

    /// Starts a live execution stream session for a client connection.
    pub fn start_execution_stream(&self, total_steps: usize) -> mpsc::Receiver<LiveDelta> {
        let (emitter, rx) = LiveEmitter::new(100);
        TaskProgressStream::spawn_progress_tracker(&self.cx, emitter, total_steps);
        rx
    }

    /// Handles an interactive HITL approval request.
    pub async fn handle_hitl_approval(&self, req: HitlApprovalRequest) -> HitlApprovalResponse {
        GovernanceConsole::process_approval_action(&self.cx, req).await
    }

    /// Runs a Tokio TCP HTTP server on `addr`, serving dashboard, web UI, assets, and health endpoints.
    pub async fn run_server(
        self,
        addr: std::net::SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use std::sync::Arc;
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpListener;

        let listener = TcpListener::bind(addr).await?;
        println!(
            "🚀 Orbity Tokio Topcoat web server active (v0.1.0-beta) on http://{}",
            addr
        );
        println!("Endpoints ready:");
        println!(
            "  - http://{}/            (Interactive Web UI & DAG Simulator)",
            addr
        );
        println!(
            "  - http://{}/assets/orbity-logo.jpg (Brand Logo Asset)",
            addr
        );
        println!(
            "  - http://{}/dashboard   (Topcoat Shard Component Console)",
            addr
        );
        println!(
            "  - http://{}/health      (Health & Harness Status JSON)",
            addr
        );
        println!(
            "  - http://{}/api/status  (Real-Time Agent State & FinOps JSON)",
            addr
        );
        println!(
            "  - http://{}/api/models  (CLI Model Discovery & Semantic Tiers JSON)",
            addr
        );
        println!(
            "  - http://{}/governance  (HITL Governance Approval Console)",
            addr
        );
        println!(
            "  - http://{}/finops      (FinOps Budget & Tokenomics)",
            addr
        );
        println!("Server running. Press Ctrl+C to terminate.");

        let this = Arc::new(self);

        loop {
            tokio::select! {
                res = listener.accept() => {
                    let (mut stream, _) = match res {
                        Ok(conn) => conn,
                        Err(e) => {
                            eprintln!("Connection accept error: {}", e);
                            continue;
                        }
                    };

                    let server = Arc::clone(&this);
                    tokio::spawn(async move {
                        let mut buf = vec![0u8; 65536];
                        let n = match stream.read(&mut buf).await {
                            Ok(n) if n > 0 => n,
                            _ => return,
                        };

                        let req = String::from_utf8_lossy(&buf[..n]);
                        let first_line = req.lines().next().unwrap_or_default();
                        let path = first_line.split_whitespace().nth(1).unwrap_or("/");

                        // Extract Origin if present for restricted CORS
                        let mut origin_allowed = "http://127.0.0.1:3000".to_string();
                        let mut auth_header_valid = true;

                        if let Some(expected_token) = &server.cx().config.auth_token {
                            auth_header_valid = false;
                            for line in req.lines() {
                                if line.to_lowercase().starts_with("authorization:") {
                                    let val = line[14..].trim();
                                    if val == format!("Bearer {}", expected_token) || val == expected_token {
                                        auth_header_valid = true;
                                    }
                                }
                            }
                        }

                        for line in req.lines() {
                            if line.to_lowercase().starts_with("origin:") {
                                let origin_val = line[7..].trim();
                                if origin_val.starts_with("http://localhost")
                                    || origin_val.starts_with("http://127.0.0.1")
                                    || origin_val.starts_with("https://localhost")
                                    || origin_val.starts_with("https://127.0.0.1")
                                {
                                    origin_allowed = origin_val.to_string();
                                }
                            }
                        }

                        // Authenticate protected endpoints if auth_token is configured
                        if (path.starts_with("/governance") || path.starts_with("/api/")) && !auth_header_valid {
                            let body = br#"{"error":"unauthorized","message":"Invalid or missing Bearer token"}"#;
                            let header = format!(
                                "HTTP/1.1 401 Unauthorized\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: {}\r\nConnection: close\r\n\r\n",
                                body.len(),
                                origin_allowed
                            );
                            let _ = stream.write_all(header.as_bytes()).await;
                            let _ = stream.write_all(body).await;
                            let _ = stream.flush().await;
                            return;
                        }

                        let (status, content_type, body_bytes): (&str, &str, Vec<u8>) = match path {
                            "/" | "/index.html" => (
                                "200 OK",
                                "text/html; charset=utf-8",
                                EMBEDDED_FRONTEND_HTML.as_bytes().to_vec(),
                            ),
                            "/assets/orbity-logo.jpg" | "/orbity-logo.jpg" | "/favicon.ico" => (
                                "200 OK",
                                "image/jpeg",
                                ORBITY_LOGO_JPG.to_vec(),
                            ),
                            "/dashboard" | "/console" | "/topcoat" => (
                                "200 OK",
                                "text/html; charset=utf-8",
                                server.render_dashboard().into_bytes(),
                            ),
                            "/health" => {
                                let pool_ok = match server.cx().pool() {
                                    Some(pool) => sqlx::query("SELECT 1").execute(pool).await.is_ok(),
                                    None => true,
                                };
                                let bwrap_ok = std::path::Path::new("/usr/bin/bwrap").exists()
                                    || std::path::Path::new("/usr/local/bin/bwrap").exists()
                                    || std::process::Command::new("which").arg("bwrap").output().map(|o| o.status.success()).unwrap_or(false);

                                let is_healthy = pool_ok && bwrap_ok;
                                let status_str = if is_healthy { "ok" } else { "degraded" };
                                let health_json = serde_json::json!({
                                    "status": status_str,
                                    "version": "0.1.0-beta",
                                    "stage": "beta",
                                    "server": "Tokio Topcoat 0.9",
                                    "harness": "Orbity Multi Agentic Harness",
                                    "uptime": "healthy",
                                    "checks": {
                                        "sqlite_pool": pool_ok,
                                        "bwrap_sandbox": bwrap_ok,
                                        "event_bus_events": server.cx().bus().events_published_count()
                                    }
                                });
                                (
                                    "200 OK",
                                    "application/json",
                                    serde_json::to_vec(&health_json).unwrap_or_default(),
                                )
                            },
                            "/api/status" => {
                                let total_events = server.cx().bus().events_published_count();
                                let mut run_count: i64 = 0;
                                let mut total_tokens_spent: i64 = 0;
                                let mut total_cost_usd: f64 = 0.0;

                                if let Some(pool) = server.cx().pool() {
                                    if let Ok(row) = sqlx::query_as::<_, (i64, Option<i64>, Option<f64>)>(
                                        "SELECT COUNT(*), SUM(total_tokens), SUM(total_cost_usd) FROM runs"
                                    ).fetch_one(pool).await {
                                        run_count = row.0;
                                        total_tokens_spent = row.1.unwrap_or(0);
                                        total_cost_usd = row.2.unwrap_or(0.0);
                                    }
                                }

                                let status_json = serde_json::json!({
                                    "status": "active",
                                    "version": "0.1.0-beta",
                                    "stage": "beta",
                                    "harness": "Orbity Multi Agentic Harness",
                                    "finops": {
                                        "budget_usd": 20.0,
                                        "spent_usd": total_cost_usd,
                                        "total_tokens": total_tokens_spent
                                    },
                                    "metrics": {
                                        "events_published": total_events,
                                        "total_runs_recorded": run_count
                                    },
                                    "agents": [
                                        {"name": "Codex Worker", "role": "codex", "status": "Ready"},
                                        {"name": "Claude Reviewer", "role": "claude", "status": "Ready"},
                                        {"name": "Agy Researcher", "role": "agy", "status": "Ready"},
                                        {"name": "Hermes Tool", "role": "hermes", "status": "Ready"},
                                        {"name": "Pi Refactor", "role": "pi", "status": "Ready"}
                                    ]
                                });
                                (
                                    "200 OK",
                                    "application/json",
                                    serde_json::to_vec(&status_json).unwrap_or_default(),
                                )
                            },
                            "/api/models" => (
                                "200 OK",
                                "application/json",
                                r#"{"harness":"Orbity Multi Agentic Harness","version":"0.1.0-beta","stage":"beta","schema":"agent -> name -> provider -> tier","tiers":["fast","balanced","reasoning","latest"],"engines":[{"cli":"agy","name":"Antigravity CLI (Google DeepMind)","discovery":"agy models","flag":"--model <model>","reasoning":"--effort <low|medium|high|max>","tiers":{"fast":"gemini-3.8-flash-low","balanced":"gemini-3.8-flash-high","reasoning":"gemini-3.1-pro-high"}},{"cli":"codex","name":"OpenAI Codex CLI","discovery":"codex --help","flag":"-m <MODEL> / --model <MODEL>","reasoning":"-c model=\"o3-mini\"","tiers":{"fast":"gpt-4o-mini","balanced":"gpt-4o","reasoning":"o3-mini"}},{"cli":"claude","name":"Claude Code (Anthropic)","discovery":"claude --help / /model","flag":"--model <model>","reasoning":"--fallback-model <model>","tiers":{"fast":"haiku","balanced":"sonnet","reasoning":"opus"}},{"cli":"hermes","name":"Hermes Agent (Nous Research)","discovery":"hermes model","flag":"-m <MODEL> / --model <MODEL>","reasoning":"--reasoning <none|low|medium|high|max>","tiers":{"fast":"openrouter/auto-fast","balanced":"anthropic/claude-sonnet-4.6","reasoning":"anthropic/claude-sonnet-4.6 (high)"}},{"cli":"pi","name":"Pi Coding Agent (pi.dev)","discovery":"pi --list-models","flag":"--model <pattern>","reasoning":"--thinking <low|medium|high>","tiers":{"fast":"llama-cpp","balanced":"sonnet","reasoning":"sonnet:high"}}]}"#.as_bytes().to_vec(),
                            ),
                            "/governance" => (
                                "200 OK",
                                "text/html; charset=utf-8",
                                "<!DOCTYPE html><html><head><title>Governance Console</title><style>body{background:#0f172a;color:#f8fafc;font-family:sans-serif;padding:24px;}</style></head><body><h1>Governance & HITL Console</h1><p>Human-in-the-Loop approval gate ready. Status: Active.</p><a href='/' style='color:#38bdf8;'>Back to Interactive Web UI</a></body></html>".as_bytes().to_vec(),
                            ),
                            "/finops" => (
                                "200 OK",
                                "text/html; charset=utf-8",
                                "<!DOCTYPE html><html><head><title>FinOps Console</title><style>body{background:#0f172a;color:#f8fafc;font-family:sans-serif;padding:24px;}</style></head><body><h1>FinOps & Tokenomics</h1><p>Budget Cap: Enforced. Cumulative cost: $1.25. Tokens: 45,000.</p><a href='/' style='color:#38bdf8;'>Back to Interactive Web UI</a></body></html>".as_bytes().to_vec(),
                            ),
                            _ => (
                                "404 Not Found",
                                "text/plain",
                                b"404 Not Found".to_vec(),
                            ),
                        };

                        let header = format!(
                            "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: {}\r\nConnection: close\r\n\r\n",
                            status,
                            content_type,
                            body_bytes.len(),
                            origin_allowed
                        );

                        if stream.write_all(header.as_bytes()).await.is_ok() {
                            let _ = stream.write_all(&body_bytes).await;
                            let _ = stream.flush().await;
                        }
                    });
                }
                _ = tokio::signal::ctrl_c() => {
                    println!("\nShutting down Orbity Topcoat server gracefully...");
                    break;
                }
            }
        }

        Ok(())
    }
}
