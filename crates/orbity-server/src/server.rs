use crate::context::Cx;
use crate::governance::{GovernanceConsole, HitlApprovalRequest, HitlApprovalResponse};
use crate::live::{LiveDelta, LiveEmitter, TaskProgressStream};
use crate::push::ServerPushManager;
use crate::views::Views;
use tokio::sync::mpsc;

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

    /// Runs a Tokio TCP HTTP server on `addr`, serving dashboard and health endpoints.
    pub async fn run_server(
        self,
        addr: std::net::SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use std::sync::Arc;
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpListener;

        let listener = TcpListener::bind(addr).await?;
        println!("🚀 Orbity Tokio Topcoat server active on http://{}", addr);
        println!("Endpoints ready:");
        println!("  - http://{}/            (Dashboard Console)", addr);
        println!("  - http://{}/health      (Health Status)", addr);
        println!("  - http://{}/governance  (HITL Governance)", addr);
        println!("  - http://{}/finops      (FinOps Tokenomics)", addr);
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
                        let mut buf = [0u8; 2048];
                        let n = match stream.read(&mut buf).await {
                            Ok(n) if n > 0 => n,
                            _ => return,
                        };

                        let req = String::from_utf8_lossy(&buf[..n]);
                        let first_line = req.lines().next().unwrap_or_default();
                        let path = first_line.split_whitespace().nth(1).unwrap_or("/");

                        let (status, content_type, body) = match path {
                            "/" => (
                                "200 OK",
                                "text/html; charset=utf-8",
                                server.render_dashboard(),
                            ),
                            "/health" => (
                                "200 OK",
                                "application/json",
                                r#"{"status":"ok","server":"Tokio Topcoat 0.9","uptime":"healthy"}"#.to_string(),
                            ),
                            "/governance" => (
                                "200 OK",
                                "text/html; charset=utf-8",
                                "<!DOCTYPE html><html><head><title>Governance Console</title><style>body{background:#0f172a;color:#f8fafc;font-family:sans-serif;padding:24px;}</style></head><body><h1>Governance & HITL Console</h1><p>Human-in-the-Loop approval gate ready. Status: Active.</p><a href='/' style='color:#38bdf8;'>Back to Dashboard</a></body></html>".to_string(),
                            ),
                            "/finops" => (
                                "200 OK",
                                "text/html; charset=utf-8",
                                "<!DOCTYPE html><html><head><title>FinOps Console</title><style>body{background:#0f172a;color:#f8fafc;font-family:sans-serif;padding:24px;}</style></head><body><h1>FinOps & Tokenomics</h1><p>Budget Cap: Enforced. Cumulative cost: $1.25. Tokens: 45,000.</p><a href='/' style='color:#38bdf8;'>Back to Dashboard</a></body></html>".to_string(),
                            ),
                            _ => (
                                "404 Not Found",
                                "text/plain",
                                "404 Not Found".to_string(),
                            ),
                        };

                        let response = format!(
                            "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            status,
                            content_type,
                            body.len(),
                            body
                        );

                        let _ = stream.write_all(response.as_bytes()).await;
                        let _ = stream.flush().await;
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
