use super::super::ViewerSourceIdentity;
use std::sync::atomic::{AtomicU64, Ordering};

const OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const PRIME: u64 = 0x00000100000001b3;
static NEXT_TRACE_SESSION_ID: AtomicU64 = AtomicU64::new(1);

pub(in crate::multi_format) fn start_trace_session(
    identity: &ViewerSourceIdentity,
) -> Option<super::super::debug_trace::TraceSession> {
    super::super::debug_trace::DebugTrace::enabled().then(|| {
        (
            NEXT_TRACE_SESSION_ID.fetch_add(1, Ordering::Relaxed),
            trace_source_fingerprint(identity),
        )
    })
}

pub(in crate::multi_format) fn trace_source_fingerprint(identity: &ViewerSourceIdentity) -> u64 {
    identity
        .uri
        .bytes()
        .chain(std::iter::once(0))
        .chain(identity.revision.bytes())
        .fold(OFFSET_BASIS, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(PRIME)
        })
}
