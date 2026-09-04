use std::cell::RefCell;
#[cfg(windows)]
use std::ffi::OsString;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

const TRACE_SESSION_ENV: &str = "KDV_TRACE_SESSION";
const TRACE_SOURCE_ENV: &str = "KDV_TRACE_SOURCE";
const SOURCE_FINGERPRINT_WIDTH: usize = 16;
const FNV1A_64_OFFSET: u64 = 0xcbf29ce484222325;
const FNV1A_64_PRIME: u64 = 0x00000100000001b3;
static NEXT_WORKSPACE_SESSION_ID: AtomicU64 = AtomicU64::new(1);

pub(crate) type TraceSession = (u64, u64);

thread_local! {
    static TRACE_CORRELATION: RefCell<Option<TraceCorrelation>> = const { RefCell::new(None) };
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TraceCorrelation {
    session_id: u64,
    source_fingerprint: u64,
}

impl TraceCorrelation {
    fn from_session((session_id, source_fingerprint): TraceSession) -> Self {
        Self {
            session_id,
            source_fingerprint,
        }
    }

    pub(super) fn session(&self) -> TraceSession {
        (self.session_id, self.source_fingerprint)
    }

    pub(super) fn from_environment() -> Option<Self> {
        Self::from_values(
            &std::env::var(TRACE_SESSION_ENV).ok()?,
            &std::env::var(TRACE_SOURCE_ENV).ok()?,
        )
    }

    fn from_values(session: &str, source: &str) -> Option<Self> {
        let session_id = session.parse().ok()?;
        if source.len() != SOURCE_FINGERPRINT_WIDTH
            || !source.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return None;
        }
        let source_fingerprint = u64::from_str_radix(source, 16).ok()?;
        Some(Self {
            session_id,
            source_fingerprint,
        })
    }

    pub(super) fn fields(&self) -> String {
        format!(
            "session={} source={:016x}",
            self.session_id, self.source_fingerprint
        )
    }

    pub(super) fn worker_environment(&self) -> (String, String) {
        (
            self.session_id.to_string(),
            format!("{:016x}", self.source_fingerprint),
        )
    }
}

pub(crate) struct TraceCorrelationGuard {
    previous: Option<TraceCorrelation>,
}

impl Drop for TraceCorrelationGuard {
    fn drop(&mut self) {
        TRACE_CORRELATION.with(|current| {
            current.replace(self.previous.take());
        });
    }
}

pub(super) fn session(session: TraceSession) -> TraceCorrelationGuard {
    let correlation = TraceCorrelation::from_session(session);
    let previous = TRACE_CORRELATION.with(|current| current.replace(Some(correlation)));
    TraceCorrelationGuard { previous }
}

pub(super) fn current() -> Option<TraceCorrelation> {
    TRACE_CORRELATION.with(|current| current.borrow().clone())
}

pub(super) fn current_session() -> Option<TraceSession> {
    current().map(|correlation| correlation.session())
}

pub(super) fn worker_environment() -> Option<(String, String)> {
    current().map(|correlation| correlation.worker_environment())
}

pub(super) fn workspace_session(workspace: &Path) -> TraceSession {
    let source_fingerprint = workspace
        .to_string_lossy()
        .bytes()
        .fold(FNV1A_64_OFFSET, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(FNV1A_64_PRIME)
        });
    (
        NEXT_WORKSPACE_SESSION_ID.fetch_add(1, Ordering::Relaxed),
        source_fingerprint,
    )
}

#[cfg(windows)]
pub(super) fn configure_worker_environment(environment: &mut Vec<(OsString, OsString)>) {
    if let Some(correlation) = current() {
        let (session, source) = correlation.worker_environment();
        environment.retain(|(name, _)| {
            name.as_os_str() != std::ffi::OsStr::new(TRACE_SESSION_ENV)
                && name.as_os_str() != std::ffi::OsStr::new(TRACE_SOURCE_ENV)
        });
        environment.push((OsString::from(TRACE_SESSION_ENV), OsString::from(session)));
        environment.push((OsString::from(TRACE_SOURCE_ENV), OsString::from(source)));
    }
}

#[cfg(test)]
mod tests {
    use super::TraceCorrelation;

    #[test]
    fn worker_correlation_accepts_only_the_expected_serialized_fields() {
        assert_eq!(
            Some(TraceCorrelation {
                session_id: 42,
                source_fingerprint: 0x0123_4567_89ab_cdef,
            }),
            TraceCorrelation::from_values("42", "0123456789abcdef")
        );
        assert!(TraceCorrelation::from_values("session", "0123456789abcdef").is_none());
        assert!(TraceCorrelation::from_values("42", "0123456789abcde").is_none());
        assert!(TraceCorrelation::from_values("42", "0123456789abcdeg").is_none());
    }
}
