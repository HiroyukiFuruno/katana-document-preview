use std::time::{Duration, Instant};

#[cfg(not(windows))]
pub(crate) struct SpreadsheetProcessOwner {
    pub(crate) child: Option<std::process::Child>,
}

#[cfg(not(windows))]
impl SpreadsheetProcessOwner {
    pub(crate) fn status(&mut self) -> Option<i64> {
        self.child
            .as_mut()
            .and_then(|child| child.try_wait().ok().flatten())
            .and_then(|status| status.code())
            .map(i64::from)
    }

    pub(crate) fn terminate(&mut self) {
        if let Some(child) = self.child.as_mut() {
            let _ = child.kill();
        }
    }

    pub(crate) fn finish(&mut self, timeout: Duration) {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            if self.status().is_some() {
                self.child = None;
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        self.terminate();
        if let Some(mut child) = self.child.take() {
            let _ = child.wait();
        }
    }
}

#[cfg(all(test, not(windows)))]
mod tests {
    use super::SpreadsheetProcessOwner;
    use std::time::Duration;

    #[test]
    fn owner_finishes_exited_and_forcibly_terminated_processes()
    -> Result<(), Box<dyn std::error::Error>> {
        let child = std::process::Command::new("/usr/bin/true").spawn()?;
        let mut exited = SpreadsheetProcessOwner { child: Some(child) };
        exited.finish(Duration::from_secs(1));
        assert!(exited.child.is_none());

        let child = std::process::Command::new("/bin/sleep").arg("5").spawn()?;
        let mut running = SpreadsheetProcessOwner { child: Some(child) };
        running.finish(Duration::ZERO);
        assert!(running.child.is_none());
        Ok(())
    }
}

#[cfg(windows)]
pub(crate) struct SpreadsheetProcessOwner {
    pub(crate) child: Option<rappct::LaunchedIo>,
}

#[cfg(windows)]
impl SpreadsheetProcessOwner {
    pub(crate) fn status(&mut self) -> Option<i64> {
        None
    }

    pub(crate) fn terminate(&mut self) {
        self.child = None;
    }

    pub(crate) fn finish(&mut self, timeout: Duration) {
        if let Some(child) = self.child.take() {
            let _ = child.wait(Some(timeout));
        }
    }
}
