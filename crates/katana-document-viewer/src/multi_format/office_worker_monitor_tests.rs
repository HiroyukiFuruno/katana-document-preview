use super::MacOsMemoryMonitor;
use std::time::{Duration, Instant};

#[test]
fn monitor_marks_and_terminates_a_process_above_the_limit() {
    let child = std::process::Command::new("/bin/sleep").arg("5").spawn();
    assert!(child.is_ok());
    if let Ok(mut child) = child {
        let monitor = MacOsMemoryMonitor::start(child.id(), 0);
        let deadline = Instant::now() + Duration::from_secs(2);
        while !monitor.exceeded() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(monitor.finish());
        let _ = child.wait();
    }
}

#[test]
fn monitor_finishes_without_exceeding_for_stopped_and_missing_processes() {
    let monitor = MacOsMemoryMonitor::start(std::process::id(), usize::MAX);
    std::thread::sleep(Duration::from_millis(20));
    assert!(!monitor.finish());
    let monitor = MacOsMemoryMonitor::start(u32::MAX, usize::MAX);
    std::thread::sleep(Duration::from_millis(20));
    assert!(!monitor.finish());
}
