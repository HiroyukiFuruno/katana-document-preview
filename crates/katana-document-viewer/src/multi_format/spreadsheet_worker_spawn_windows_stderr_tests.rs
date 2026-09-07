use super::{forward_debug_stderr, forward_stderr, forward_stderr_chunks, stderr_unavailable};
use crate::multi_format::OfficeWorkerError;
use std::io::Read;
use std::sync::{Arc, Mutex};

#[test]
fn stderr_forwarding_preserves_trace_lines() {
    let mut source = &b"spreadsheet.runtime_init elapsed_ms=4\n"[..];
    let mut output = Vec::new();

    forward_stderr(&mut source, &mut output);

    assert_eq!(
        b"spreadsheet.runtime_init elapsed_ms=4\n",
        output.as_slice()
    );
}

#[test]
fn debug_stderr_forwarding_releases_the_sink_before_the_next_read() -> std::io::Result<()> {
    let lock = Arc::new(Mutex::new(()));
    let mut source = LockCheckingReader::new(
        vec![b"spreadsheet.runtime_init\n", b"spreadsheet.frame\n"],
        Arc::clone(&lock),
    );
    let mut output = Vec::new();

    forward_stderr_chunks(&mut source, |chunk| {
        let guard = lock
            .lock()
            .map_err(|_| std::io::Error::other("stderr sink lock poisoned"))?;
        output.extend_from_slice(chunk);
        drop(guard);
        Ok(())
    })?;

    assert_eq!(
        b"spreadsheet.runtime_init\nspreadsheet.frame\n",
        output.as_slice()
    );
    Ok(())
}

#[test]
fn debug_stderr_forwarding_writes_each_chunk() -> std::io::Result<()> {
    let mut source = &b"spreadsheet.runtime_init elapsed_ms=4\n"[..];

    forward_debug_stderr(&mut source)
}

#[test]
fn debug_stderr_forwarding_reports_a_still_locked_sink() -> std::io::Result<()> {
    let lock = Arc::new(Mutex::new(()));
    let mut source = LockCheckingReader::new(
        vec![b"spreadsheet.runtime_init\n", b"spreadsheet.frame\n"],
        Arc::clone(&lock),
    );
    let guard = lock
        .lock()
        .map_err(|_| std::io::Error::other("stderr sink lock poisoned"))?;
    let result = forward_stderr_chunks(&mut source, |_| Ok(()));
    drop(guard);

    assert!(matches!(
        result,
        Err(ref error)
            if error.kind() == std::io::ErrorKind::Other
                && error.to_string() == "stderr sink lock must be released before the next read"
    ));
    Ok(())
}

#[test]
fn unavailable_stderr_is_a_typed_protocol_error() {
    assert!(matches!(
        stderr_unavailable(),
        OfficeWorkerError::Protocol { .. }
    ));
}

struct LockCheckingReader {
    chunks: Vec<&'static [u8]>,
    next_chunk: usize,
    lock: Arc<Mutex<()>>,
}

impl LockCheckingReader {
    fn new(chunks: Vec<&'static [u8]>, lock: Arc<Mutex<()>>) -> Self {
        Self {
            chunks,
            next_chunk: 0,
            lock,
        }
    }
}

impl Read for LockCheckingReader {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        if self.next_chunk > 0 {
            let guard = self.lock.try_lock().map_err(|_| {
                std::io::Error::other("stderr sink lock must be released before the next read")
            })?;
            drop(guard);
        }
        let Some(chunk) = self.chunks.get(self.next_chunk) else {
            return Ok(0);
        };
        buffer[..chunk.len()].copy_from_slice(chunk);
        self.next_chunk += 1;
        Ok(chunk.len())
    }
}
