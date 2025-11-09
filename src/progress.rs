//! Progress tracking and display

use indicatif::{ProgressBar, ProgressStyle};
use std::time::Instant;

/// Progress tracker for CSV loading
pub struct ProgressTracker {
    bar: ProgressBar,
    start_time: Instant,
    total_rows: Option<u64>,
}

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new(total_rows: Option<u64>, quiet: bool) -> Self {
        let bar = if quiet {
            ProgressBar::hidden()
        } else if let Some(total) = total_rows {
            ProgressBar::new(total)
        } else {
            ProgressBar::new_spinner()
        };

        let style = if total_rows.is_some() {
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} rows ({percent}%) | {per_sec} | ETA: {eta}")
                .unwrap()
                .progress_chars("=>-")
        } else {
            ProgressStyle::default_spinner()
                .template("[{elapsed_precise}] {spinner} {pos} rows | {per_sec}")
                .unwrap()
        };

        bar.set_style(style);

        Self {
            bar,
            start_time: Instant::now(),
            total_rows,
        }
    }

    /// Update progress with row count
    pub fn update(&self, rows_processed: u64) {
        self.bar.set_position(rows_processed);
    }

    /// Increment progress by delta
    pub fn inc(&self, delta: u64) {
        self.bar.inc(delta);
    }

    /// Set a status message
    pub fn set_message(&self, msg: String) {
        self.bar.set_message(msg);
    }

    /// Finish and show completion message
    pub fn finish(&self) {
        let elapsed = self.start_time.elapsed();
        let rows = self.bar.position();

        let throughput = if elapsed.as_secs() > 0 {
            rows / elapsed.as_secs()
        } else {
            rows
        };

        let message = format!(
            "Completed! {} rows in {:.2}s ({} rows/sec)",
            rows,
            elapsed.as_secs_f64(),
            throughput
        );

        self.bar.finish_with_message(message);
    }

    /// Finish with error message
    pub fn finish_with_error(&self, error: &str) {
        self.bar.finish_with_message(format!("Failed: {}", error));
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    /// Get current throughput (rows/sec)
    pub fn throughput(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.bar.position() as f64 / elapsed
        } else {
            0.0
        }
    }
}

impl Drop for ProgressTracker {
    fn drop(&mut self) {
        // Ensure progress bar is cleared on drop
        if !self.bar.is_finished() {
            self.bar.finish_and_clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_tracker_creation() {
        let tracker = ProgressTracker::new(Some(100), true);
        assert_eq!(tracker.total_rows, Some(100));
    }

    #[test]
    fn test_progress_update() {
        let tracker = ProgressTracker::new(Some(100), true);
        tracker.update(50);
        assert_eq!(tracker.bar.position(), 50);
    }

    #[test]
    fn test_progress_increment() {
        let tracker = ProgressTracker::new(None, true);
        tracker.inc(10);
        tracker.inc(5);
        assert_eq!(tracker.bar.position(), 15);
    }

    #[test]
    fn test_throughput_calculation() {
        let tracker = ProgressTracker::new(None, true);
        tracker.update(100);
        std::thread::sleep(std::time::Duration::from_millis(100));
        let throughput = tracker.throughput();
        assert!(throughput > 0.0);
    }
}
