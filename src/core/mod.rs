pub mod mihomo_runner;
pub mod real_speedtest;
pub mod speedtest;
pub mod statistics;

pub use mihomo_runner::MihomoRunner;
pub use real_speedtest::RealSpeedTester;
pub use speedtest::{SpeedTestConfig, SpeedTestResult, SpeedTester};
pub use statistics::StatisticalAnalysis;
