use super::spreadsheet_worker_protocol::{
    MAX_SPREADSHEET_RESPONSE_BYTES, SpreadsheetWorkerResponse,
};
use std::io::{BufRead, BufReader, Read};
use std::sync::mpsc::Receiver;
use std::thread::JoinHandle;

pub(crate) struct SpreadsheetResponses {
    pub(crate) receiver: Receiver<Result<SpreadsheetWorkerResponse, String>>,
    pub(crate) worker: JoinHandle<()>,
}

pub(crate) struct SpreadsheetResponseReader;

impl SpreadsheetResponseReader {
    pub(crate) fn spawn(output: Box<dyn Read + Send>) -> SpreadsheetResponses {
        let (sender, receiver) = std::sync::mpsc::channel();
        let worker = std::thread::spawn(move || {
            let mut reader = BufReader::new(output);
            loop {
                let mut bytes = Vec::new();
                match read_response(&mut reader, &mut bytes) {
                    Ok(None) => break,
                    Ok(Some(response)) => {
                        if sender.send(response).is_err() {
                            break;
                        }
                    }
                    Err(message) => {
                        let _ = sender.send(Err(message));
                        break;
                    }
                }
            }
        });
        SpreadsheetResponses { receiver, worker }
    }
}

fn read_response(
    reader: &mut BufReader<Box<dyn Read + Send>>,
    bytes: &mut Vec<u8>,
) -> Result<Option<Result<SpreadsheetWorkerResponse, String>>, String> {
    let read = match reader.read_until(b'\n', bytes) {
        Ok(read) => read,
        Err(error) => return Err(response_read_failure(error)),
    };
    if read == 0 {
        return Ok(None);
    }
    if bytes.len() > MAX_SPREADSHEET_RESPONSE_BYTES {
        return Err("spreadsheet response exceeds its byte limit".to_owned());
    }
    let response = serde_json::from_slice(bytes).map_err(invalid_response);
    Ok(Some(response))
}

fn response_read_failure(error: std::io::Error) -> String {
    format!("spreadsheet response read failed: {error}")
}

fn invalid_response(error: serde_json::Error) -> String {
    format!("invalid spreadsheet response: {error}")
}

#[cfg(test)]
mod tests {
    use super::{
        MAX_SPREADSHEET_RESPONSE_BYTES, SpreadsheetResponseReader, SpreadsheetWorkerResponse,
        read_response,
    };
    use std::io::{BufReader, Cursor, Read};
    use std::sync::{Arc, Barrier};

    #[test]
    fn reader_reports_invalid_oversized_and_io_failed_frames()
    -> Result<(), Box<dyn std::error::Error>> {
        let invalid = SpreadsheetResponseReader::spawn(Box::new(Cursor::new(b"not json\n")));
        assert!(invalid.receiver.recv()?.is_err());
        invalid
            .worker
            .join()
            .map_err(|_| "reader thread panicked")?;

        let oversized = SpreadsheetResponseReader::spawn(Box::new(Cursor::new(vec![
            b'x';
            MAX_SPREADSHEET_RESPONSE_BYTES
                + 1
        ])));
        assert!(oversized.receiver.recv()?.is_err());
        oversized
            .worker
            .join()
            .map_err(|_| "reader thread panicked")?;

        let mut reader = BufReader::new(Box::new(FailingReader) as Box<dyn Read + Send>);
        assert!(read_response(&mut reader, &mut Vec::new()).is_err());
        Ok(())
    }

    #[test]
    fn reader_stops_when_the_receiver_is_gone() -> Result<(), Box<dyn std::error::Error>> {
        let mut bytes = serde_json::to_vec(&SpreadsheetWorkerResponse::Stopped)?;
        bytes.push(b'\n');
        let read_barrier = Arc::new(Barrier::new(2));
        let responses = SpreadsheetResponseReader::spawn(Box::new(GatedReader::new(
            bytes,
            Arc::clone(&read_barrier),
        )));
        drop(responses.receiver);
        read_barrier.wait();
        responses
            .worker
            .join()
            .map_err(|_| "reader thread panicked")?;
        Ok(())
    }

    struct GatedReader {
        inner: Cursor<Vec<u8>>,
        read_barrier: Arc<Barrier>,
        waiting: bool,
    }

    impl GatedReader {
        fn new(bytes: Vec<u8>, read_barrier: Arc<Barrier>) -> Self {
            Self {
                inner: Cursor::new(bytes),
                read_barrier,
                waiting: true,
            }
        }
    }

    impl Read for GatedReader {
        fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
            if self.waiting {
                self.waiting = false;
                self.read_barrier.wait();
            }
            self.inner.read(bytes)
        }
    }

    struct FailingReader;

    impl Read for FailingReader {
        fn read(&mut self, _bytes: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("read failed"))
        }
    }
}
