mod multi_format {
    #[path = "debug_trace.rs"]
    mod debug_trace;
    #[path = "office_worker_constraints.rs"]
    mod office_worker_constraints;
    #[path = "office_worker_input.rs"]
    mod office_worker_input;
    mod office_worker_protocol {
        pub(super) use super::office_worker_input::INPUT_NAME;
    }
    #[path = "spreadsheet_engine.rs"]
    mod spreadsheet_engine;
    #[path = "spreadsheet_engine_cell.rs"]
    mod spreadsheet_engine_cell;
    #[path = "spreadsheet_engine_cell_border.rs"]
    mod spreadsheet_engine_cell_border;
    #[path = "spreadsheet_engine_sheet.rs"]
    mod spreadsheet_engine_sheet;
    #[path = "spreadsheet_engine_support.rs"]
    mod spreadsheet_engine_support;
    #[path = "spreadsheet_filter_engine.rs"]
    mod spreadsheet_filter_engine;
    #[cfg(test)]
    #[path = "spreadsheet_filter_test_support.rs"]
    mod spreadsheet_filter_test_support;
    #[path = "spreadsheet_filter_xml.rs"]
    mod spreadsheet_filter_xml;
    #[path = "spreadsheet_filter_xml_parser.rs"]
    mod spreadsheet_filter_xml_parser;
    #[cfg(test)]
    #[path = "spreadsheet_filter_xml_tests.rs"]
    mod spreadsheet_filter_xml_tests;
    #[path = "spreadsheet_streaming.rs"]
    mod spreadsheet_streaming;
    #[path = "spreadsheet_streaming_cell_reader.rs"]
    mod spreadsheet_streaming_cell_reader;
    #[path = "spreadsheet_streaming_cell_types.rs"]
    mod spreadsheet_streaming_cell_types;
    #[path = "spreadsheet_streaming_cells.rs"]
    mod spreadsheet_streaming_cells;
    #[path = "spreadsheet_streaming_sheet_metadata.rs"]
    mod spreadsheet_streaming_sheet_metadata;
    #[path = "spreadsheet_streaming_xml.rs"]
    mod spreadsheet_streaming_xml;
    #[path = "spreadsheet_streaming_xml_values.rs"]
    mod spreadsheet_streaming_xml_values;
    #[path = "spreadsheet_worker_arguments.rs"]
    mod spreadsheet_worker_arguments;
    #[path = "spreadsheet_worker_artifact.rs"]
    mod spreadsheet_worker_artifact;
    #[path = "spreadsheet_worker_entrypoint.rs"]
    mod spreadsheet_worker_entrypoint;
    #[path = "spreadsheet_worker_protocol.rs"]
    mod spreadsheet_worker_protocol;

    pub use spreadsheet_worker_artifact::*;

    pub struct WorkerApp;

    impl WorkerApp {
        pub fn run() -> i32 {
            let _ = SpreadsheetViewerLimits::strict();
            let _ = spreadsheet_worker_protocol::SPREADSHEET_MODE;
            spreadsheet_worker_entrypoint::SpreadsheetWorkerEntrypoint::run_from_env()
        }
    }
}

fn main() {
    std::process::exit(multi_format::WorkerApp::run());
}
