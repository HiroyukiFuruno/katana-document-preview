use katana_document_viewer::{
    BinaryDocumentSource, DocumentResourceSnapshot, DocumentSession, DocumentSessionConfig,
    DocumentViewport, OfficeDocumentFormat, OfficeDocumentSource, OfficeWorkerConfig, ViewerSource,
    ViewerSourceIdentity,
};
use std::path::{Path, PathBuf};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const CYCLES: usize = 10;

#[test]
fn ten_mixed_document_cycles_return_resources_to_baseline() -> TestResult {
    let baseline = DocumentResourceSnapshot::capture();
    for cycle in 0..CYCLES {
        run_pdf_cycle(cycle)?;
        run_spreadsheet_cycle(cycle)?;
        run_presentation_cycle(cycle)?;
        assert_eq!(baseline, DocumentResourceSnapshot::capture());
    }
    Ok(())
}

fn run_pdf_cycle(cycle: usize) -> TestResult {
    let bytes = include_bytes!("../../../assets/reference/katana/pdf/sample.pdf").to_vec();
    let source = BinaryDocumentSource::new(
        ViewerSourceIdentity::new("file:///cycle.pdf", format!("cycle:{cycle}")),
        "application/pdf",
        bytes,
    );
    let mut session = DocumentSession::open(
        ViewerSource::Pdf(source),
        DocumentSessionConfig::new(DocumentViewport::new(320, 240)),
    )?;
    let _ = session.frame()?;
    session.close();
    Ok(())
}

fn run_spreadsheet_cycle(cycle: usize) -> TestResult {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/fixtures/multi-format/representative.xlsx");
    let source = OfficeDocumentSource::new(
        ViewerSourceIdentity::new("file:///cycle.xlsx", format!("cycle:{cycle}")),
        OfficeDocumentFormat::Xlsx,
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        std::fs::read(fixture)?,
    );
    let config = DocumentSessionConfig::new(DocumentViewport::new(320, 240))
        .office_worker(OfficeWorkerConfig::new(worker_binary_path()?));
    let mut session = DocumentSession::open(ViewerSource::Office(source), config)?;
    let _ = session.frame()?;
    session.close();
    Ok(())
}

fn run_presentation_cycle(cycle: usize) -> TestResult {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/fixtures/multi-format/representative.pptx");
    let source = OfficeDocumentSource::new(
        ViewerSourceIdentity::new("file:///cycle.pptx", format!("cycle:{cycle}")),
        OfficeDocumentFormat::Pptx,
        "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        std::fs::read(fixture)?,
    );
    let config = DocumentSessionConfig::new(DocumentViewport::new(320, 240))
        .office_worker(OfficeWorkerConfig::new(worker_binary_path()?));
    let mut session = DocumentSession::open(ViewerSource::Office(source), config)?;
    let _ = session.frame()?;
    session.close();
    Ok(())
}

fn worker_binary_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let current_exe = std::env::current_exe()?;
    let deps = current_exe.parent().ok_or("test binary has no parent")?;
    let worker = deps
        .parent()
        .ok_or("test binary has no target")?
        .join("kdv-office-worker");
    #[cfg(windows)]
    let worker = worker.with_extension("exe");
    Ok(worker)
}
