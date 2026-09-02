use super::super::debug_trace::DebugTrace;

pub(super) fn apply_runtime_constraints(
    arguments: &super::WorkerArguments,
    apply_constraints: super::ConstraintApplier,
) -> Result<(), (String, String)> {
    let _runtime_init = DebugTrace::start("office.runtime_init");
    apply_constraints(
        &arguments.workspace,
        arguments.max_memory_bytes,
        arguments.max_cpu_seconds,
    )
}
