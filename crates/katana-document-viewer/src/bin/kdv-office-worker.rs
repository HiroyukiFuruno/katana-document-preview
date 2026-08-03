use katana_document_viewer::OfficeWorkerEntrypoint;

fn main() {
    std::process::exit(OfficeWorkerEntrypoint::run_from_env());
}
