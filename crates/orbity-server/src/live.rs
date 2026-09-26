use crate::context::Cx;
use crate::views::ViewHtml;
use tokio::sync::mpsc;

/// A live stream delta emitted from the server to the client browser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiveDelta {
    /// Initial skeleton or suspense block.
    Skeleton(String),
    /// Incremental DOM morph update for a specific element or shard.
    Morph { selector: String, html: String },
    /// Appended log or event entry.
    Append { container_id: String, html: String },
    /// Final stream completion marker.
    Complete(String),
}

/// Channel for emitting UI updates from server components via `emit!` semantics.
#[derive(Clone)]
pub struct LiveEmitter {
    tx: mpsc::Sender<LiveDelta>,
}

impl LiveEmitter {
    pub fn new(capacity: usize) -> (Self, mpsc::Receiver<LiveDelta>) {
        let (tx, rx) = mpsc::channel(capacity);
        (Self { tx }, rx)
    }

    /// Emits a skeleton or loading placeholder.
    pub async fn emit_skeleton(
        &self,
        html: impl Into<String>,
    ) -> Result<(), mpsc::error::SendError<LiveDelta>> {
        self.tx.send(LiveDelta::Skeleton(html.into())).await
    }

    /// Emits a DOM morph update for a target selector.
    pub async fn emit_morph(
        &self,
        selector: impl Into<String>,
        html: impl Into<String>,
    ) -> Result<(), mpsc::error::SendError<LiveDelta>> {
        self.tx
            .send(LiveDelta::Morph {
                selector: selector.into(),
                html: html.into(),
            })
            .await
    }

    /// Emits an appended HTML element into a container.
    pub async fn emit_append(
        &self,
        container_id: impl Into<String>,
        html: impl Into<String>,
    ) -> Result<(), mpsc::error::SendError<LiveDelta>> {
        self.tx
            .send(LiveDelta::Append {
                container_id: container_id.into(),
                html: html.into(),
            })
            .await
    }

    /// Emits stream completion.
    pub async fn emit_complete(
        &self,
        final_html: impl Into<String>,
    ) -> Result<(), mpsc::error::SendError<LiveDelta>> {
        self.tx.send(LiveDelta::Complete(final_html.into())).await
    }
}

/// Helper struct for streaming graph task execution progress smoothly.
pub struct TaskProgressStream;

impl TaskProgressStream {
    /// Creates and spawns a reactive progress stream emitting status deltas.
    pub fn spawn_progress_tracker(
        cx: &Cx,
        emitter: LiveEmitter,
        total_steps: usize,
    ) -> tokio::task::JoinHandle<()> {
        let mut rx = cx.bus().subscribe();

        tokio::spawn(async move {
            let _ = emitter
                .emit_skeleton(
                    "<div class=\"topcoat-skeleton\">Executing multi-agent graph...</div>",
                )
                .await;

            let mut step_count = 0;
            while let Ok(envelope) = rx.recv().await {
                step_count += 1;
                let pct = ((step_count as f64 / total_steps.max(1) as f64) * 100.0).min(100.0);

                let card =
                    ViewHtml::new(
                        "div",
                        format!(
                        "<div class=\"progress-update\"><span>Step {} ({}%)</span><p>{}</p></div>",
                        step_count, pct as u32, envelope.event.event_type_name()
                    ),
                    )
                    .with_class("progress-card");

                let _ = emitter
                    .emit_morph(
                        "#progress-bar",
                        format!("<div style=\"width: {}%\"></div>", pct as u32),
                    )
                    .await;
                let _ = emitter
                    .emit_append("#event-stream-container", card.render())
                    .await;

                if step_count >= total_steps {
                    let _ = emitter
                        .emit_complete(
                            "<div class=\"alert-success\">Workflow completed successfully!</div>",
                        )
                        .await;
                    break;
                }
            }
        })
    }
}
