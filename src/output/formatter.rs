use crate::core::SpeedTestResult;
use comfy_table::{Cell, Color, ContentArrangement, Table, presets::UTF8_FULL};
use serde_json;

/// Formatter for speed test results
pub struct ResultFormatter {
    json_output: bool,
    use_colors: bool,
}

impl ResultFormatter {
    /// Create a new result formatter
    pub fn new(json_output: bool, use_colors: bool) -> Self {
        Self {
            json_output,
            use_colors,
        }
    }

    /// Format results for display
    pub fn format_results(&self, results: &[SpeedTestResult]) -> String {
        if self.json_output {
            self.format_json(results)
        } else {
            self.format_table(results)
        }
    }

    /// Format results as JSON
    fn format_json(&self, results: &[SpeedTestResult]) -> String {
        serde_json::to_string_pretty(results)
            .unwrap_or_else(|_| "Error formatting JSON".to_string())
    }

    /// Format results as a table
    fn format_table(&self, results: &[SpeedTestResult]) -> String {
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                "Proxy Name",
                "Type",
                "Latency",
                "Jitter",
                "Loss %",
                "Download",
                "Upload",
                "Status",
            ]);

        for result in results {
            let latency_cell = self.format_latency_cell(result);
            let jitter_cell = self.format_jitter_cell(result);
            let download_cell =
                self.format_speed_cell(result.download_speed, 10.0 * 1024.0 * 1024.0);
            let upload_cell = self.format_speed_cell(result.upload_speed, 5.0 * 1024.0 * 1024.0);
            let status_cell = self.format_status_cell(result);

            table.add_row(vec![
                Cell::new(&result.proxy_name),
                Cell::new(result.proxy_type.to_string()),
                latency_cell,
                jitter_cell,
                Cell::new(format!("{:.1}", result.packet_loss)),
                download_cell,
                upload_cell,
                status_cell,
            ]);
        }

        table.to_string()
    }

    /// Format latency cell with color coding
    fn format_latency_cell(&self, result: &SpeedTestResult) -> Cell {
        match result.latency {
            Some(latency) => {
                let latency_ms = latency.as_millis();
                let text = format!("{latency_ms}ms");

                if !self.use_colors {
                    return Cell::new(text);
                }

                let cell = Cell::new(text);
                if latency_ms < 100 {
                    cell.fg(Color::Green)
                } else if latency_ms < 300 {
                    cell.fg(Color::Yellow)
                } else if latency_ms < 800 {
                    cell.fg(Color::Magenta)
                } else {
                    cell.fg(Color::Red)
                }
            }
            None => {
                let cell = Cell::new("Failed");
                if self.use_colors {
                    cell.fg(Color::Red)
                } else {
                    cell
                }
            }
        }
    }

    /// Format jitter cell
    fn format_jitter_cell(&self, result: &SpeedTestResult) -> Cell {
        match result.jitter {
            Some(jitter) => {
                let jitter_ms = jitter.as_millis();
                let text = format!("{jitter_ms}ms");

                if !self.use_colors {
                    return Cell::new(text);
                }

                let cell = Cell::new(text);
                if jitter_ms < 50 {
                    cell.fg(Color::Green)
                } else if jitter_ms < 100 {
                    cell.fg(Color::Yellow)
                } else {
                    cell.fg(Color::Red)
                }
            }
            None => Cell::new("-"),
        }
    }

    /// Format speed cell with color coding
    fn format_speed_cell(&self, speed: f64, good_threshold: f64) -> Cell {
        if speed <= 0.0 {
            let cell = Cell::new("Failed");
            return if self.use_colors {
                cell.fg(Color::Red)
            } else {
                cell
            };
        }

        let speed_mbps = speed / (1024.0 * 1024.0);
        let text = format!("{speed_mbps:.2} MB/s");

        if !self.use_colors {
            return Cell::new(text);
        }

        let cell = Cell::new(text);
        let good_threshold_mbps = good_threshold / (1024.0 * 1024.0);

        if speed_mbps >= good_threshold_mbps {
            cell.fg(Color::Green)
        } else if speed_mbps >= good_threshold_mbps * 0.5 {
            cell.fg(Color::Yellow)
        } else {
            cell.fg(Color::Red)
        }
    }

    /// Format status cell
    fn format_status_cell(&self, result: &SpeedTestResult) -> Cell {
        let (text, color) = if result.is_successful() {
            ("Success", Color::Green)
        } else {
            ("Failed", Color::Red)
        };

        let cell = Cell::new(text);
        if self.use_colors {
            cell.fg(color)
        } else {
            cell
        }
    }

    /// Format a summary of the results
    pub fn format_summary(&self, results: &[SpeedTestResult]) -> String {
        let total = results.len();
        let successful = results.iter().filter(|r| r.is_successful()).count();
        let failed = total - successful;

        let avg_latency = if successful > 0 {
            let total_latency: u128 = results
                .iter()
                .filter_map(|r| r.latency)
                .map(|l| l.as_millis())
                .sum();
            total_latency / successful as u128
        } else {
            0
        };

        let avg_download_speed = if successful > 0 {
            let total_speed: f64 = results.iter().map(|r| r.download_speed).sum();
            (total_speed / successful as f64) / (1024.0 * 1024.0)
        } else {
            0.0
        };

        format!(
            "\n📊 Summary:\n  Total: {total} | ✅ Success: {successful} | ❌ Failed: {failed}\n  📈 Avg Latency: {avg_latency}ms | 📊 Avg Download: {avg_download_speed:.2} MB/s"
        )
    }
}
