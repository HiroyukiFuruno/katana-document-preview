use super::{SpreadsheetEngineError, SpreadsheetEngineSupport};

#[test]
fn engine_support_rejects_invalid_sizes_and_indices() {
    assert_eq!(0.0, SpreadsheetEngineSupport::track_size(f64::NAN));
    assert_eq!(0.0, SpreadsheetEngineSupport::track_size(-1.0));
    assert!(matches!(
        SpreadsheetEngineSupport::check_limit("cells", 2, 1),
        Err(SpreadsheetEngineError::ResourceLimit { .. })
    ));
    assert!(SpreadsheetEngineSupport::zero_based(0).is_err());
    assert!(SpreadsheetEngineSupport::engine_index(usize::MAX).is_err());
}

#[test]
fn engine_support_preserves_external_error_context() {
    let conversion = usize::try_from(-1_i32);
    assert!(conversion.is_err());
    if let Err(error) = conversion {
        assert!(matches!(
            SpreadsheetEngineSupport::model_error(error),
            SpreadsheetEngineError::Model(_)
        ));
    }
    assert!(matches!(
        SpreadsheetEngineSupport::engine_error("engine failed".to_owned()),
        SpreadsheetEngineError::Model(_)
    ));
}
