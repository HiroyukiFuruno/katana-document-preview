#[cfg(target_os = "macos")]
const MEMORY_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);

#[cfg(target_os = "macos")]
pub(super) struct MacOsMemoryMonitor {
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    exceeded: std::sync::Arc<std::sync::atomic::AtomicBool>,
    worker: std::thread::JoinHandle<()>,
}

#[cfg(target_os = "macos")]
impl MacOsMemoryMonitor {
    pub(super) fn start(process_id: u32, limit: usize) -> Self {
        use std::sync::Arc;
        use std::sync::atomic::AtomicBool;

        let stop = Arc::new(AtomicBool::new(false));
        let exceeded = Arc::new(AtomicBool::new(false));
        let worker = Self::spawn(process_id, limit, Arc::clone(&stop), Arc::clone(&exceeded));
        Self {
            stop,
            exceeded,
            worker,
        }
    }

    #[must_use]
    pub(super) fn exceeded(&self) -> bool {
        use std::sync::atomic::Ordering;

        self.exceeded.load(Ordering::Acquire)
    }

    pub(super) fn finish(self) -> bool {
        use std::sync::atomic::Ordering;

        self.stop.store(true, Ordering::Release);
        let _ = self.worker.join();
        self.exceeded.load(Ordering::Acquire)
    }

    fn spawn(
        process_id: u32,
        limit: usize,
        stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
        exceeded: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || Self::monitor(process_id, limit, &stop, &exceeded))
    }

    fn monitor(
        process_id: u32,
        limit: usize,
        stop: &std::sync::atomic::AtomicBool,
        exceeded: &std::sync::atomic::AtomicBool,
    ) {
        use std::sync::atomic::Ordering;
        use sysinfo::{Pid, ProcessesToUpdate, System};

        let process_id = Pid::from_u32(process_id);
        let mut system = System::new();
        while !stop.load(Ordering::Acquire) {
            system.refresh_processes(ProcessesToUpdate::Some(&[process_id]), true);
            let Some(process) = system.process(process_id) else {
                break;
            };
            if process.memory() > limit as u64 {
                exceeded.store(true, Ordering::Release);
                let _ = process.kill();
                break;
            }
            std::thread::sleep(MEMORY_POLL_INTERVAL);
        }
    }
}

#[cfg(all(test, target_os = "macos"))]
#[path = "office_worker_monitor_tests.rs"]
mod tests;
