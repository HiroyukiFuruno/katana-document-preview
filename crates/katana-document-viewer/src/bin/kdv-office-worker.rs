use katana_document_viewer::OfficeWorkerEntrypoint;

fn main() {
    std::process::exit(OfficeWorkerEntrypoint::run_from_env());
}

#[cfg(test)]
#[path = "kdv_office_worker/tests.rs"]
mod tests;
