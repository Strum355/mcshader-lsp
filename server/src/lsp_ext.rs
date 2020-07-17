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

pub enum ConfigUpdate {}

impl Notification for ConfigUpdate {
    type Params = ConfigUpdateParams;
    const METHOD: &'static str = "mc-glsl/updateConfig";
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct ConfigUpdateParams {
    pub kv: Vec<KV>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct KV {
    key: String,
    value: String
}