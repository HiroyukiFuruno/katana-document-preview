use super::OfficeWorkerError;
#[cfg(windows)]
use std::io::BufReader;
use std::io::{Read, Write};

#[cfg(windows)]
pub(super) fn spawn_stderr_reader(
    stderr: std::fs::File,
    debug_enabled: bool,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || drain_stderr(stderr, debug_enabled))
}

#[cfg(any(windows, test))]
pub(super) fn stderr_unavailable() -> OfficeWorkerError {
    OfficeWorkerError::protocol("spreadsheet worker stderr is unavailable".to_owned())
}

#[cfg(windows)]
fn drain_stderr(stderr: std::fs::File, debug_enabled: bool) {
    let mut source = BufReader::new(stderr);
    if debug_enabled {
        forward_debug_stderr(&mut source);
    } else {
        let mut sink = std::io::sink();
        forward_stderr(&mut source, &mut sink);
    }
}

#[cfg(any(windows, test))]
fn forward_debug_stderr(source: &mut impl Read) -> std::io::Result<()> {
    forward_stderr_chunks(source, |chunk| {
        let mut parent_stderr = std::io::stderr().lock();
        parent_stderr.write_all(chunk)
    })
}

fn forward_stderr_chunks(
    source: &mut impl Read,
    mut write_chunk: impl FnMut(&[u8]) -> std::io::Result<()>,
) -> std::io::Result<()> {
    let mut buffer = [0_u8; 8 * 1024];
    loop {
        let bytes_read = source.read(&mut buffer)?;
        if bytes_read == 0 {
            return Ok(());
        }
        write_chunk(&buffer[..bytes_read])?;
    }
}

fn forward_stderr(source: &mut impl Read, target: &mut impl Write) {
    // stderr は診断専用なので、転送失敗で worker protocol を壊さない。
    let _ = std::io::copy(source, target);
}

#[cfg(test)]
#[path = "spreadsheet_worker_spawn_windows_stderr_tests.rs"]
mod tests;
