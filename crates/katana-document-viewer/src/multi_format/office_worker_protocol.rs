use serde::{Deserialize, Serialize};

pub(super) const INPUT_NAME: &str = "input.office";
pub(super) const OUTPUT_NAME: &str = "output.pdf";
pub(super) const RESPONSE_NAME: &str = "response.json";
pub(super) const MAX_RESPONSE_BYTES: u64 = 64 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub(super) enum OfficeWorkerResponse {
    Completed { warnings: Vec<String> },
    Failed { stage: String, message: String },
}
