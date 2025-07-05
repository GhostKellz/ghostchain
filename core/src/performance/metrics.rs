use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize, Deserializer, Serializer};

// Custom serialization for Duration
mod duration_serde {
    use super::*;
    
    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_millis() as u64)
    }
    
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

/// Performance metrics collector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub operations: HashMap<String, OperationMetrics>,
    pub network_metrics: NetworkMetrics,
    pub storage_metrics: StorageMetrics,
    pub contract_metrics: ContractMetrics,
    #[serde(skip)]
    pub start_time: Option<Instant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationMetrics {
    pub total_calls: u64,
    #[serde(with = "duration_serde")]
    pub total_duration: Duration,
    #[serde(with = "duration_serde")]
    pub min_duration: Duration,
    #[serde(with = "duration_serde")]
    pub max_duration: Duration,
    #[serde(with = "duration_serde")]
    pub avg_duration: Duration,
    pub error_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connections_established: u64,
    pub connections_closed: u64,
    pub network_errors: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMetrics {
    pub reads: u64,
    pub writes: u64,
    pub deletes: u64,
    pub bytes_read: u64,
    pub bytes_written: u64,
    pub storage_errors: u64,
    #[serde(with = "duration_serde")]
    pub average_read_time: Duration,
    #[serde(with = "duration_serde")]
    pub average_write_time: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractMetrics {
    pub deployments: u64,
    pub executions: u64,
    pub gas_used: u128,
    pub execution_errors: u64,
    #[serde(with = "duration_serde")]
    pub average_execution_time: Duration,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            cache_hits: 0,
            cache_misses: 0,
            operations: HashMap::new(),
            network_metrics: NetworkMetrics::new(),
            storage_metrics: StorageMetrics::new(),
            contract_metrics: ContractMetrics::new(),
            start_time: Some(Instant::now()),
        }
    }

    pub fn record_operation(&mut self, operation: &str, duration: Duration) {
        let metrics = self.operations.entry(operation.to_string()).or_insert_with(|| {
            OperationMetrics {
                total_calls: 0,
                total_duration: Duration::from_secs(0),
                min_duration: Duration::from_secs(u64::MAX),
                max_duration: Duration::from_secs(0),
                avg_duration: Duration::from_secs(0),
                error_count: 0,
            }
        });

        metrics.total_calls += 1;
        metrics.total_duration += duration;
        
        if duration < metrics.min_duration {
            metrics.min_duration = duration;
        }
        
        if duration > metrics.max_duration {
            metrics.max_duration = duration;
        }
        
        metrics.avg_duration = metrics.total_duration / metrics.total_calls as u32;
    }

    pub fn record_operation_error(&mut self, operation: &str) {
        let metrics = self.operations.entry(operation.to_string()).or_insert_with(|| {
            OperationMetrics {
                total_calls: 0,
                total_duration: Duration::from_secs(0),
                min_duration: Duration::from_secs(u64::MAX),
                max_duration: Duration::from_secs(0),
                avg_duration: Duration::from_secs(0),
                error_count: 0,
            }
        });

        metrics.error_count += 1;
    }

    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }

    pub fn uptime(&self) -> Duration {
        if let Some(start_time) = self.start_time {
            start_time.elapsed()
        } else {
            Duration::from_secs(0)
        }
    }

    pub fn operations_per_second(&self) -> f64 {
        let uptime = self.uptime();
        if uptime.as_secs() == 0 {
            0.0
        } else {
            let total_operations: u64 = self.operations.values().map(|m| m.total_calls).sum();
            total_operations as f64 / uptime.as_secs() as f64
        }
    }

    pub fn get_operation_summary(&self) -> Vec<(String, OperationSummary)> {
        self.operations
            .iter()
            .map(|(name, metrics)| {
                (
                    name.clone(),
                    OperationSummary {
                        total_calls: metrics.total_calls,
                        avg_duration_ms: metrics.avg_duration.as_millis() as f64,
                        error_rate: if metrics.total_calls > 0 {
                            metrics.error_count as f64 / metrics.total_calls as f64
                        } else {
                            0.0
                        },
                    },
                )
            })
            .collect()
    }

    pub fn reset(&mut self) {
        self.cache_hits = 0;
        self.cache_misses = 0;
        self.operations.clear();
        self.network_metrics = NetworkMetrics::new();
        self.storage_metrics = StorageMetrics::new();
        self.contract_metrics = ContractMetrics::new();
        self.start_time = Some(Instant::now());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationSummary {
    pub total_calls: u64,
    pub avg_duration_ms: f64,
    pub error_rate: f64,
}

impl NetworkMetrics {
    pub fn new() -> Self {
        Self {
            messages_sent: 0,
            messages_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            connections_established: 0,
            connections_closed: 0,
            network_errors: 0,
        }
    }

    pub fn record_message_sent(&mut self, bytes: u64) {
        self.messages_sent += 1;
        self.bytes_sent += bytes;
    }

    pub fn record_message_received(&mut self, bytes: u64) {
        self.messages_received += 1;
        self.bytes_received += bytes;
    }

    pub fn record_connection_established(&mut self) {
        self.connections_established += 1;
    }

    pub fn record_connection_closed(&mut self) {
        self.connections_closed += 1;
    }

    pub fn record_network_error(&mut self) {
        self.network_errors += 1;
    }

    pub fn throughput_mbps(&self, uptime: Duration) -> f64 {
        let total_bytes = self.bytes_sent + self.bytes_received;
        let seconds = uptime.as_secs_f64();
        if seconds > 0.0 {
            (total_bytes as f64 * 8.0) / (seconds * 1_000_000.0)
        } else {
            0.0
        }
    }
}

impl StorageMetrics {
    pub fn new() -> Self {
        Self {
            reads: 0,
            writes: 0,
            deletes: 0,
            bytes_read: 0,
            bytes_written: 0,
            storage_errors: 0,
            average_read_time: Duration::from_secs(0),
            average_write_time: Duration::from_secs(0),
        }
    }

    pub fn record_read(&mut self, bytes: u64, duration: Duration) {
        self.reads += 1;
        self.bytes_read += bytes;
        self.average_read_time = if self.reads == 1 {
            duration
        } else {
            Duration::from_nanos(
                (self.average_read_time.as_nanos() as u64 * (self.reads - 1) + duration.as_nanos() as u64) / self.reads
            )
        };
    }

    pub fn record_write(&mut self, bytes: u64, duration: Duration) {
        self.writes += 1;
        self.bytes_written += bytes;
        self.average_write_time = if self.writes == 1 {
            duration
        } else {
            Duration::from_nanos(
                (self.average_write_time.as_nanos() as u64 * (self.writes - 1) + duration.as_nanos() as u64) / self.writes
            )
        };
    }

    pub fn record_delete(&mut self) {
        self.deletes += 1;
    }

    pub fn record_storage_error(&mut self) {
        self.storage_errors += 1;
    }

    pub fn read_throughput_mbps(&self, uptime: Duration) -> f64 {
        let seconds = uptime.as_secs_f64();
        if seconds > 0.0 {
            (self.bytes_read as f64 * 8.0) / (seconds * 1_000_000.0)
        } else {
            0.0
        }
    }

    pub fn write_throughput_mbps(&self, uptime: Duration) -> f64 {
        let seconds = uptime.as_secs_f64();
        if seconds > 0.0 {
            (self.bytes_written as f64 * 8.0) / (seconds * 1_000_000.0)
        } else {
            0.0
        }
    }
}

impl ContractMetrics {
    pub fn new() -> Self {
        Self {
            deployments: 0,
            executions: 0,
            gas_used: 0,
            execution_errors: 0,
            average_execution_time: Duration::from_secs(0),
        }
    }

    pub fn record_deployment(&mut self) {
        self.deployments += 1;
    }

    pub fn record_execution(&mut self, gas_used: u128, duration: Duration) {
        self.executions += 1;
        self.gas_used += gas_used;
        self.average_execution_time = if self.executions == 1 {
            duration
        } else {
            Duration::from_nanos(
                (self.average_execution_time.as_nanos() as u64 * (self.executions - 1) + duration.as_nanos() as u64) / self.executions
            )
        };
    }

    pub fn record_execution_error(&mut self) {
        self.execution_errors += 1;
    }

    pub fn gas_per_second(&self, uptime: Duration) -> f64 {
        let seconds = uptime.as_secs_f64();
        if seconds > 0.0 {
            self.gas_used as f64 / seconds
        } else {
            0.0
        }
    }

    pub fn execution_success_rate(&self) -> f64 {
        let total_attempts = self.executions + self.execution_errors;
        if total_attempts > 0 {
            self.executions as f64 / total_attempts as f64
        } else {
            0.0
        }
    }
}

/// Metrics reporter for outputting performance data
pub struct MetricsReporter;

impl MetricsReporter {
    pub fn generate_report(metrics: &PerformanceMetrics) -> String {
        let mut report = String::new();
        
        report.push_str("=== GhostChain Performance Report ===\n\n");
        
        // System metrics
        report.push_str(&format!("Uptime: {:?}\n", metrics.uptime()));
        report.push_str(&format!("Operations/sec: {:.2}\n", metrics.operations_per_second()));
        report.push_str(&format!("Cache Hit Rate: {:.2}%\n", metrics.cache_hit_rate() * 100.0));
        report.push_str("\n");
        
        // Operation summary
        report.push_str("=== Operation Summary ===\n");
        for (name, summary) in metrics.get_operation_summary() {
            report.push_str(&format!(
                "{}: {} calls, {:.2}ms avg, {:.2}% error rate\n",
                name,
                summary.total_calls,
                summary.avg_duration_ms,
                summary.error_rate * 100.0
            ));
        }
        report.push_str("\n");
        
        // Network metrics
        let network = &metrics.network_metrics;
        report.push_str("=== Network Metrics ===\n");
        report.push_str(&format!("Messages Sent: {}\n", network.messages_sent));
        report.push_str(&format!("Messages Received: {}\n", network.messages_received));
        report.push_str(&format!("Bytes Sent: {}\n", network.bytes_sent));
        report.push_str(&format!("Bytes Received: {}\n", network.bytes_received));
        report.push_str(&format!("Throughput: {:.2} Mbps\n", network.throughput_mbps(metrics.uptime())));
        report.push_str("\n");
        
        // Storage metrics
        let storage = &metrics.storage_metrics;
        report.push_str("=== Storage Metrics ===\n");
        report.push_str(&format!("Reads: {}\n", storage.reads));
        report.push_str(&format!("Writes: {}\n", storage.writes));
        report.push_str(&format!("Deletes: {}\n", storage.deletes));
        report.push_str(&format!("Read Throughput: {:.2} Mbps\n", storage.read_throughput_mbps(metrics.uptime())));
        report.push_str(&format!("Write Throughput: {:.2} Mbps\n", storage.write_throughput_mbps(metrics.uptime())));
        report.push_str("\n");
        
        // Contract metrics
        let contracts = &metrics.contract_metrics;
        report.push_str("=== Contract Metrics ===\n");
        report.push_str(&format!("Deployments: {}\n", contracts.deployments));
        report.push_str(&format!("Executions: {}\n", contracts.executions));
        report.push_str(&format!("Gas Used: {}\n", contracts.gas_used));
        report.push_str(&format!("Gas/sec: {:.2}\n", contracts.gas_per_second(metrics.uptime())));
        report.push_str(&format!("Success Rate: {:.2}%\n", contracts.execution_success_rate() * 100.0));
        
        report
    }
    
    pub fn generate_json_report(metrics: &PerformanceMetrics) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(metrics)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_performance_metrics_basic() {
        let mut metrics = PerformanceMetrics::new();
        
        metrics.record_operation("test_op", Duration::from_millis(100));
        metrics.record_operation("test_op", Duration::from_millis(200));
        
        let summary = metrics.get_operation_summary();
        assert_eq!(summary.len(), 1);
        assert_eq!(summary[0].1.total_calls, 2);
        assert_eq!(summary[0].1.avg_duration_ms, 150.0);
    }

    #[test]
    fn test_cache_hit_rate() {
        let mut metrics = PerformanceMetrics::new();
        
        metrics.cache_hits = 80;
        metrics.cache_misses = 20;
        
        assert_eq!(metrics.cache_hit_rate(), 0.8);
    }

    #[test]
    fn test_network_metrics() {
        let mut metrics = NetworkMetrics::new();
        
        metrics.record_message_sent(1024);
        metrics.record_message_received(2048);
        
        assert_eq!(metrics.messages_sent, 1);
        assert_eq!(metrics.messages_received, 1);
        assert_eq!(metrics.bytes_sent, 1024);
        assert_eq!(metrics.bytes_received, 2048);
    }

    #[test]
    fn test_storage_metrics() {
        let mut metrics = StorageMetrics::new();
        
        metrics.record_read(512, Duration::from_millis(10));
        metrics.record_write(1024, Duration::from_millis(20));
        
        assert_eq!(metrics.reads, 1);
        assert_eq!(metrics.writes, 1);
        assert_eq!(metrics.bytes_read, 512);
        assert_eq!(metrics.bytes_written, 1024);
    }

    #[test]
    fn test_contract_metrics() {
        let mut metrics = ContractMetrics::new();
        
        metrics.record_deployment();
        metrics.record_execution(21000, Duration::from_millis(5));
        
        assert_eq!(metrics.deployments, 1);
        assert_eq!(metrics.executions, 1);
        assert_eq!(metrics.gas_used, 21000);
    }

    #[test]
    fn test_metrics_reporter() {
        let metrics = PerformanceMetrics::new();
        
        let report = MetricsReporter::generate_report(&metrics);
        assert!(report.contains("GhostChain Performance Report"));
        
        let json_report = MetricsReporter::generate_json_report(&metrics).unwrap();
        assert!(json_report.contains("cache_hits"));
    }
}