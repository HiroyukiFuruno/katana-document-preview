use crate::multi_format::office_worker_protocol::{OfficeWorkerResponse, RESPONSE_NAME};
use std::path::Path;

type ResponseEncoder = fn(&OfficeWorkerResponse) -> Result<Vec<u8>, serde_json::Error>;

pub(super) fn write_response(
    workspace: &Path,
    response: &OfficeWorkerResponse,
    exit_code: i32,
) -> i32 {
    write_response_with(workspace, response, exit_code, encode_response)
}

fn encode_response(response: &OfficeWorkerResponse) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(response)
}

pub(super) fn write_response_with(
    workspace: &Path,
    response: &OfficeWorkerResponse,
    exit_code: i32,
    encoder: ResponseEncoder,
) -> i32 {
    let bytes = match encoder(response) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("KDV office worker response encoding failed: {error}");
            return super::EXIT_FAILURE;
        }
    };
    if let Err(error) = std::fs::write(workspace.join(RESPONSE_NAME), bytes) {
        eprintln!("KDV office worker response write failed: {error}");
        return super::EXIT_FAILURE;
    }
    exit_code
}
