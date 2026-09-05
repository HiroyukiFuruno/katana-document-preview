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

#[cfg(windows)]
fn forward_debug_stderr(source: &mut impl Read) {
    let _ = forward_stderr_chunks(source, |chunk| {
        let mut parent_stderr = std::io::stderr().lock();
        parent_stderr.write_all(chunk)
    });
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
mod tests {
    use super::{forward_stderr, forward_stderr_chunks, stderr_unavailable};
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
}
