pub(super) fn input_failure(error: std::io::Error) -> (String, String) {
    failure("input", error.to_string())
}

pub(super) fn output_failure(error: std::io::Error) -> (String, String) {
    failure("output", error.to_string())
}

pub(super) fn engine_failure(error: office2pdf::error::ConvertError) -> (String, String) {
    failure("engine", error.to_string())
}

pub(super) fn failure(stage: &str, message: String) -> (String, String) {
    (stage.to_owned(), message)
}
