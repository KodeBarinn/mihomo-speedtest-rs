use crate::core::SpeedTestResult;
use indicatif::{ProgressBar, ProgressStyle};

/// Progress bar for speed testing
pub struct SpeedTestProgress {
    bar: ProgressBar,
}

impl SpeedTestProgress {
    /// Create a new progress bar
    pub fn new(total: u64) -> Self {
        let bar = ProgressBar::new(total);
        bar.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
                )
                .unwrap()
                .progress_chars("#>-"),
        );
        bar.set_message("Initializing...");

        Self { bar }
    }

    /// Update progress with a new result
    pub fn update(&self, result: &SpeedTestResult) {
        self.bar.inc(1);

        let status = if result.is_successful() {
            format!("✓ {} ({}ms)", result.proxy_name, result.format_latency())
        } else {
            format!("✗ {} (Failed)", result.proxy_name)
        };

        self.bar.set_message(status);
    }

    /// Set a custom message
    pub fn set_message(&self, msg: &str) {
        self.bar.set_message(msg.to_string());
    }

    /// Finish the progress bar
    pub fn finish_with_message(&self, msg: &str) {
        self.bar.finish_with_message(msg.to_string());
    }

    /// Clear the progress bar (not available in all versions)
    pub fn clear(&self) {
        // Clear is not available in all versions of indicatif
        // self.bar.clear();
    }
}

impl Drop for SpeedTestProgress {
    fn drop(&mut self) {
        self.bar.finish_and_clear();
    }
}
