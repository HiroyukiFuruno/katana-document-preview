use crate::multi_format::spreadsheet_worker_protocol::{
    MAX_SPREADSHEET_RESPONSE_BYTES, SpreadsheetWorkerRequest, SpreadsheetWorkerResponse,
};
use std::io::{BufRead, Write};

pub(super) fn read_request(
    reader: &mut dyn BufRead,
    max_bytes: usize,
) -> Result<SpreadsheetWorkerRequest, String> {
    let mut bytes = Vec::new();
    let read = reader
        .read_until(b'\n', &mut bytes)
        .map_err(protocol_read_failure)?;
    if read == 0 {
        return Err("protocol input closed before shutdown".to_owned());
    }
    if bytes.len() > max_bytes {
        return Err("protocol request exceeds its byte limit".to_owned());
    }
    serde_json::from_slice(&bytes).map_err(invalid_request)
}

pub(super) fn write_response(
    writer: &mut dyn Write,
    response: &SpreadsheetWorkerResponse,
) -> Result<(), String> {
    write_response_limited(writer, response, MAX_SPREADSHEET_RESPONSE_BYTES)
}

pub(super) fn write_response_limited(
    writer: &mut dyn Write,
    response: &SpreadsheetWorkerResponse,
    max_bytes: usize,
) -> Result<(), String> {
    let bytes = serde_json::to_vec(response).map_err(response_encoding_failure)?;
    write_encoded_response(writer, &bytes, max_bytes)
}

pub(super) fn write_encoded_response(
    writer: &mut dyn Write,
    bytes: &[u8],
    max_bytes: usize,
) -> Result<(), String> {
    if bytes.len() > max_bytes {
        return Err("protocol response exceeds its byte limit".to_owned());
    }
    writer
        .write_all(bytes)
        .and_then(|()| writer.write_all(b"\n"))
        .and_then(|()| writer.flush())
        .map_err(protocol_write_failure)
}

pub(super) fn protocol_read_failure(error: std::io::Error) -> String {
    format!("protocol read failed: {error}")
}

pub(super) fn invalid_request(error: serde_json::Error) -> String {
    format!("invalid request: {error}")
}

pub(super) fn response_encoding_failure(error: serde_json::Error) -> String {
    format!("response encoding: {error}")
}

pub(super) fn protocol_write_failure(error: std::io::Error) -> String {
    format!("protocol write failed: {error}")
}
