use crate::context::Cx;
use crate::views::ViewHtml;
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitlApprovalRequest {
    pub execution_id: String,
    pub node_id: String,
    pub prompt: String,
    pub approved: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitlApprovalResponse {
    pub success: bool,
    pub decision: String,
    pub message: String,
}

pub struct GovernanceConsole;

impl GovernanceConsole {
    /// Evaluates or records an interactive approval button click from the UI (`@click`).
    pub async fn process_approval_action(
        _cx: &Cx,
        req: HitlApprovalRequest,
    ) -> HitlApprovalResponse {
        let (decision, msg) = if req.approved {
            (
                "approved",
                format!("Approved step '{}' on execution {}", req.node_id, req.execution_id),
            )
        } else {
            (
                "rejected",
                format!(
                    "Rejected step '{}' on execution {}. Reason: {}",
                    req.node_id,
                    req.execution_id,
                    req.reason.as_deref().unwrap_or("User rejected via Web Console")
                ),
            )
        };

        HitlApprovalResponse {
            success: true,
            decision: decision.to_string(),
            message: msg,
        }
    }

    /// Renders an interactive HITL modal component for the browser.
    pub fn render_hitl_prompt_modal(execution_id: &str, node_id: &str, prompt: &str) -> ViewHtml {
        let html = format!(
            "<div class=\"modal hitl-modal\">\
                <div class=\"modal-content\">\
                    <h3>Human-in-the-Loop Approval Required</h3>\
                    <p class=\"prompt-text\">{}</p>\
                    <div class=\"modal-actions\">\
                        <button class=\"btn btn-success\" data-action=\"approve\" data-exec=\"{}\" data-node=\"{}\">Approve</button>\
                        <button class=\"btn btn-danger\" data-action=\"reject\" data-exec=\"{}\" data-node=\"{}\">Reject</button>\
                    </div>\
                </div>\
            </div>",
            prompt, execution_id, node_id, execution_id, node_id
        );
        ViewHtml::new("div", html).with_class("hitl-overlay")
    }
}
