use std::time::Instant;

pub(crate) struct DebugTrace {
    stage: &'static str,
    started_at: Option<Instant>,
}

impl DebugTrace {
    #[must_use]
    pub(crate) fn start(stage: &'static str) -> Self {
        Self {
            stage,
            started_at: Self::enabled().then(Instant::now),
        }
    }

    pub(crate) fn enabled() -> bool {
        std::env::var("DEBUG")
            .ok()
            .is_some_and(|value| debug_value_enabled(&value))
    }

    pub(crate) fn event(stage: &'static str, detail: impl std::fmt::Display) {
        if Self::enabled() {
            eprintln!("[KDV_TRACE] stage={stage} {detail}");
        }
    }
}

impl Drop for DebugTrace {
    fn drop(&mut self) {
        if let Some(started_at) = self.started_at {
            eprintln!(
                "[KDV_TRACE] stage={} elapsed_ms={}",
                self.stage,
                started_at.elapsed().as_millis()
            );
        }
    }
}

fn debug_value_enabled(value: &str) -> bool {
    value.eq_ignore_ascii_case("true") || value == "1"
}

#[cfg(test)]
mod tests {
    use super::debug_value_enabled;

    #[test]
    fn debug_trace_requires_an_explicit_true_value() {
        assert!(debug_value_enabled("true"));
        assert!(debug_value_enabled("TRUE"));
        assert!(debug_value_enabled("1"));
        assert!(!debug_value_enabled("false"));
        assert!(!debug_value_enabled(""));
    }
}
