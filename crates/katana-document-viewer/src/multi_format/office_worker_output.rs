use super::office_worker_parent::OfficeWorkerError;
use super::office_worker_protocol::{
    MAX_RESPONSE_BYTES, OUTPUT_NAME, OfficeWorkerResponse, RESPONSE_NAME,
};
use std::path::Path;

pub(crate) struct OfficeWorkerOutputReader;

impl OfficeWorkerOutputReader {
    pub(crate) fn read_response(
        workspace: &Path,
    ) -> Result<OfficeWorkerResponse, OfficeWorkerError> {
        let path = workspace.join(RESPONSE_NAME);
        let metadata = regular_file_metadata(&path)?;
        if metadata.len() > MAX_RESPONSE_BYTES {
            return Err(protocol_message(format!(
                "worker response exceeds {MAX_RESPONSE_BYTES} bytes"
            )));
        }
        let bytes = std::fs::read(path).map_err(protocol_io)?;
        serde_json::from_slice(&bytes).map_err(protocol_json)
    }

    pub(crate) fn read_pdf(
        workspace: &Path,
        max_output_bytes: u64,
    ) -> Result<Vec<u8>, OfficeWorkerError> {
        let path = workspace.join(OUTPUT_NAME);
        let metadata = regular_file_metadata(&path)?;
        if metadata.len() > max_output_bytes {
            return Err(OfficeWorkerError::OutputLimitExceeded {
                actual: metadata.len(),
                limit: max_output_bytes,
            });
        }
        let bytes = std::fs::read(path).map_err(protocol_io)?;
        if !bytes.starts_with(b"%PDF-") {
            return Err(protocol_message(
                "worker output does not contain a PDF signature".to_owned(),
            ));
        }
        Ok(bytes)
    }
}

fn regular_file_metadata(path: &Path) -> Result<std::fs::Metadata, OfficeWorkerError> {
    let metadata = std::fs::symlink_metadata(path).map_err(protocol_io)?;
    if metadata.file_type().is_file() {
        return Ok(metadata);
    }
    Err(protocol_message(format!(
        "worker output `{}` is not a regular file",
        path.display()
    )))
}

fn protocol_message(message: String) -> OfficeWorkerError {
    OfficeWorkerError::Protocol { message }
}

fn protocol_io(error: std::io::Error) -> OfficeWorkerError {
    protocol_message(error.to_string())
}

fn protocol_json(error: serde_json::Error) -> OfficeWorkerError {
    protocol_message(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::{OfficeWorkerOutputReader, protocol_io};
    use crate::multi_format::OfficeWorkerError;
    use crate::multi_format::office_worker_protocol::{
        MAX_RESPONSE_BYTES, OUTPUT_NAME, RESPONSE_NAME,
    };

    #[test]
    fn response_reader_rejects_missing_and_oversized_outputs()
    -> Result<(), Box<dyn std::error::Error>> {
        assert!(matches!(
            protocol_io(std::io::Error::other("missing")),
            OfficeWorkerError::Protocol { .. }
        ));
        let missing = tempfile::tempdir()?;
        assert!(matches!(
            OfficeWorkerOutputReader::read_response(missing.path()),
            Err(OfficeWorkerError::Protocol { .. })
        ));

        let oversized = tempfile::tempdir()?;
        let response = std::fs::File::create(oversized.path().join(RESPONSE_NAME))?;
        response.set_len(MAX_RESPONSE_BYTES + 1)?;
        assert!(matches!(
            OfficeWorkerOutputReader::read_response(oversized.path()),
            Err(OfficeWorkerError::Protocol { .. })
        ));
        Ok(())
    }

    #[test]
    fn response_reader_rejects_invalid_and_non_regular_outputs()
    -> Result<(), Box<dyn std::error::Error>> {
        let invalid = tempfile::tempdir()?;
        std::fs::write(invalid.path().join(RESPONSE_NAME), b"not json")?;
        assert!(matches!(
            OfficeWorkerOutputReader::read_response(invalid.path()),
            Err(OfficeWorkerError::Protocol { .. })
        ));

        let directory = tempfile::tempdir()?;
        std::fs::create_dir(directory.path().join(RESPONSE_NAME))?;
        assert!(matches!(
            OfficeWorkerOutputReader::read_response(directory.path()),
            Err(OfficeWorkerError::Protocol { .. })
        ));
        Ok(())
    }

    #[test]
    fn pdf_reader_rejects_oversized_and_unsigned_outputs() -> Result<(), Box<dyn std::error::Error>>
    {
        let oversized = tempfile::tempdir()?;
        std::fs::write(oversized.path().join(OUTPUT_NAME), b"%PDF-large")?;
        assert_eq!(
            Err(OfficeWorkerError::OutputLimitExceeded {
                actual: 10,
                limit: 1,
            }),
            OfficeWorkerOutputReader::read_pdf(oversized.path(), 1)
        );

        let unsigned = tempfile::tempdir()?;
        std::fs::write(unsigned.path().join(OUTPUT_NAME), b"not a pdf")?;
        assert!(matches!(
            OfficeWorkerOutputReader::read_pdf(unsigned.path(), 1024),
            Err(OfficeWorkerError::Protocol { .. })
        ));
        Ok(())
    }
}
