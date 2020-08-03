use rust_lsp::lsp_types::notification::Notification;
use serde::{Deserialize, Serialize};

pub enum Status {}

impl Notification for Status {
    type Params = StatusParams;
    const METHOD: &'static str = "mc-glsl/status";
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct StatusParams {
    pub status: String,
    pub message: Option<String>,
    pub icon: Option<String>,
}