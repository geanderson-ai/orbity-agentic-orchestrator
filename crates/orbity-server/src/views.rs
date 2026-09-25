//! Reactive Topcoat Views, client signals, and server-side DOM-morphing shard components.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::watch;

/// A reactive signal holding a value and notifying subscribers when updated without roundtrips.
#[derive(Clone)]
pub struct Signal<T: Clone + Send + Sync + 'static> {
    tx: Arc<watch::Sender<T>>,
    rx: watch::Receiver<T>,
    version: Arc<AtomicUsize>,
}

impl<T: Clone + Send + Sync + 'static> Signal<T> {
    pub fn new(initial: T) -> Self {
        let (tx, rx) = watch::channel(initial);
        Self {
            tx: Arc::new(tx),
            rx,
            version: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Reads the current value of the signal.
    pub fn get(&self) -> T {
        self.rx.borrow().clone()
    }

    /// Mutates the signal value and increments the reactivity version.
    pub fn set(&self, val: T) {
        let _ = self.tx.send(val);
        self.version.fetch_add(1, Ordering::SeqCst);
    }

    /// Reactive version counter for shard morphing triggers.
    pub fn version(&self) -> usize {
        self.version.load(Ordering::SeqCst)
    }
}

/// Helper function mirroring `topcoat::signal(cx, initial)`
pub fn signal<T: Clone + Send + Sync + 'static>(initial: T) -> Signal<T> {
    Signal::new(initial)
}

/// Rendered HTML View produced by a Topcoat view component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewHtml {
    pub tag: String,
    pub classes: Vec<String>,
    pub html: String,
}

impl ViewHtml {
    pub fn new(tag: impl Into<String>, html: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            classes: Vec::new(),
            html: html.into(),
        }
    }

    pub fn with_class(mut self, class: impl Into<String>) -> Self {
        self.classes.push(class.into());
        self
    }

    pub fn render(&self) -> String {
        let class_attr = if self.classes.is_empty() {
            String::new()
        } else {
            format!(" class=\"{}\"", self.classes.join(" "))
        };
        format!("<{}{}>{}</{}>", self.tag, class_attr, self.html, self.tag)
    }
}

/// Shard component representing a dynamic fragment re-rendered on the server
/// and morphed into the client DOM over long-lived connection.
#[derive(Clone)]
pub struct ShardComponent {
    pub name: String,
    pub shard_id: String,
    render_fn: Arc<dyn Fn() -> ViewHtml + Send + Sync>,
}

impl ShardComponent {
    pub fn new<F>(name: impl Into<String>, shard_id: impl Into<String>, render_fn: F) -> Self
    where
        F: Fn() -> ViewHtml + Send + Sync + 'static,
    {
        Self {
            name: name.into(),
            shard_id: shard_id.into(),
            render_fn: Arc::new(render_fn),
        }
    }

    /// Renders the shard with a Topcoat morphing wrapper: `<div data-shard="...">...</div>`
    pub fn render_shard(&self) -> String {
        let inner = (self.render_fn)().render();
        format!(
            "<div data-topcoat-shard=\"{}\" data-shard-name=\"{}\">{}</div>",
            self.shard_id, self.name, inner
        )
    }
}

/// Built-in view builders for Agent Cards, FinOps Widgets, and Graph Progress.
pub struct Views;

impl Views {
    pub fn agent_card(name: &str, cli: &str, state: &str, tokens: u64, cost_usd: f64) -> ViewHtml {
        let content = format!(
            "<div class=\"agent-header\"><h3>{}</h3><span class=\"badge badge-{}\">{}</span></div>\
             <div class=\"agent-meta\"><span>CLI: {}</span><span>Tokens: {}</span><span>Cost: ${:.4}</span></div>",
            name, state.to_lowercase(), state, cli, tokens, cost_usd
        );
        ViewHtml::new("div", content).with_class("card agent-card")
    }

    pub fn finops_widget(budget_limit: f64, total_spent: f64, total_tokens: u64) -> ViewHtml {
        let pct = if budget_limit > 0.0 { (total_spent / budget_limit) * 100.0 } else { 0.0 };
        let content = format!(
            "<div class=\"finops-header\"><h4>FinOps Real-Time Budget</h4></div>\
             <div class=\"progress-bar\"><div class=\"fill\" style=\"width: {:.1}%\"></div></div>\
             <div class=\"finops-stats\"><span>Spent: ${:.4} / ${:.4} ({:.1}%)</span><span>Tokens: {}</span></div>",
            pct, total_spent, budget_limit, pct, total_tokens
        );
        ViewHtml::new("div", content).with_class("widget finops-widget")
    }

    pub fn audit_chain_badge(is_valid: bool, block_count: usize) -> ViewHtml {
        let (status_class, status_text) = if is_valid {
            ("verified", "SHA-256 Chain Verified")
        } else {
            ("tampered", "Tampering Detected!")
        };
        let content = format!(
            "<span class=\"audit-status {}\">{} ({} blocks)</span>",
            status_class, status_text, block_count
        );
        ViewHtml::new("div", content).with_class("audit-badge")
    }
}
