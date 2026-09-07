use super::{
    OfficeDocumentFormat, OfficeDocumentSource, OfficePreflightLimits, OfficeWorkerConfig,
};
use std::path::PathBuf;
use std::time::Duration;

const FNV1A_128_OFFSET: u128 = 0x6c62_272e_07bb_0142_62b8_2175_6295_c58d;
const FNV1A_128_PRIME: u128 = 0x0000_0000_0100_0000_0000_0000_0000_013b;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct OfficeConversionKey {
    content_revision: String,
    content_fingerprint: u128,
    content_bytes: usize,
    format: OfficeDocumentFormat,
    worker_executable: PathBuf,
    timeout: Duration,
    max_memory_bytes: usize,
    max_output_bytes: u64,
    preflight_limits: OfficePreflightLimits,
}

impl OfficeConversionKey {
    pub(super) fn new(source: &OfficeDocumentSource, config: &OfficeWorkerConfig) -> Self {
        Self {
            content_revision: source.identity.revision.clone(),
            content_fingerprint: content_fingerprint(&source.bytes),
            content_bytes: source.bytes.len(),
            format: source.format,
            worker_executable: config.executable.clone(),
            timeout: config.timeout,
            max_memory_bytes: config.max_memory_bytes,
            max_output_bytes: config.max_output_bytes,
            preflight_limits: config.preflight_limits,
        }
    }

    pub(super) const fn content_bytes(&self) -> usize {
        self.content_bytes
    }
}

fn content_fingerprint(bytes: &[u8]) -> u128 {
    bytes.iter().fold(FNV1A_128_OFFSET, |fingerprint, byte| {
        (fingerprint ^ u128::from(*byte)).wrapping_mul(FNV1A_128_PRIME)
    })
}

#[cfg(test)]
mod tests {
    use super::OfficeConversionKey;
    use crate::multi_format::{
        OfficeDocumentFormat, OfficeDocumentSource, OfficeWorkerConfig, ViewerSourceIdentity,
    };
    use std::path::PathBuf;

    fn source(bytes: &[u8]) -> OfficeDocumentSource {
        OfficeDocumentSource::new(
            ViewerSourceIdentity::new("file:///key.docx", "revision-1"),
            OfficeDocumentFormat::Docx,
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            bytes.to_vec(),
        )
    }

    #[test]
    fn key_changes_with_content_format_or_worker_settings() {
        let config = OfficeWorkerConfig::new(PathBuf::from("worker-a"));
        let baseline = OfficeConversionKey::new(&source(b"a"), &config);
        assert_ne!(baseline, OfficeConversionKey::new(&source(b"b"), &config));

        let mut pptx = source(b"a");
        pptx.format = OfficeDocumentFormat::Pptx;
        assert_ne!(baseline, OfficeConversionKey::new(&pptx, &config));

        let mut changed = config.clone();
        changed.max_output_bytes += 1;
        assert_ne!(baseline, OfficeConversionKey::new(&source(b"a"), &changed));
    }
}
