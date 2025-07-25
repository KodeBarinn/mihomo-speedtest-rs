use std::time::Duration;

/// Statistical analysis utilities
pub struct StatisticalAnalysis;

impl StatisticalAnalysis {
    /// Calculate mean of a set of durations
    pub fn mean_duration(values: &[Duration]) -> Duration {
        if values.is_empty() {
            return Duration::ZERO;
        }

        let total: Duration = values.iter().sum();
        total / values.len() as u32
    }

    /// Calculate standard deviation of durations (jitter)
    pub fn std_deviation_duration(values: &[Duration], mean: Duration) -> Duration {
        if values.len() <= 1 {
            return Duration::ZERO;
        }

        let variance: f64 = values
            .iter()
            .map(|&val| {
                let diff = val.as_nanos() as f64 - mean.as_nanos() as f64;
                diff * diff
            })
            .sum::<f64>()
            / values.len() as f64;

        Duration::from_nanos(variance.sqrt() as u64)
    }

    /// Calculate packet loss percentage
    pub fn packet_loss_percentage(failed: usize, total: usize) -> f64 {
        if total == 0 {
            return 0.0;
        }
        (failed as f64 / total as f64) * 100.0
    }

    /// Calculate median of durations
    pub fn median_duration(values: &mut [Duration]) -> Option<Duration> {
        if values.is_empty() {
            return None;
        }

        values.sort();
        let len = values.len();

        if len % 2 == 0 {
            let mid1 = values[len / 2 - 1];
            let mid2 = values[len / 2];
            Some((mid1 + mid2) / 2)
        } else {
            Some(values[len / 2])
        }
    }

    /// Calculate percentile of durations
    pub fn percentile_duration(values: &mut [Duration], percentile: f64) -> Option<Duration> {
        if values.is_empty() || !(0.0..=100.0).contains(&percentile) {
            return None;
        }

        values.sort();
        let index = (percentile / 100.0 * (values.len() - 1) as f64).round() as usize;
        Some(values[index])
    }
}
