use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{interval, Instant};
use tracing::{debug, info, warn};

/// Batch processor for handling bulk operations efficiently
pub struct BatchProcessor {
    batch_size: usize,
    batch_timeout: Duration,
    pending_operations: Arc<Mutex<Vec<BatchOperation>>>,
    processor_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

#[derive(Debug, Clone)]
pub enum BatchOperation {
    BlockInsert { height: u64, data: Vec<u8> },
    AccountUpdate { address: String, data: Vec<u8> },
    TransactionStore { tx_id: String, data: Vec<u8> },
    ContractExecution { contract_id: String, input: Vec<u8> },
    NetworkMessage { peer_id: String, message: Vec<u8> },
}

pub struct BatchResult {
    pub processed: usize,
    pub failed: usize,
    pub duration: Duration,
}

impl BatchProcessor {
    pub fn new(batch_size: usize, batch_timeout: Duration) -> Self {
        Self {
            batch_size,
            batch_timeout,
            pending_operations: Arc::new(Mutex::new(Vec::new())),
            processor_handle: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn start(&self) -> Result<()> {
        let mut handle = self.processor_handle.lock().await;
        if handle.is_some() {
            return Ok(()); // Already running
        }

        let pending_operations = self.pending_operations.clone();
        let batch_size = self.batch_size;
        let batch_timeout = self.batch_timeout;

        let processor_handle = tokio::spawn(async move {
            let mut interval = interval(batch_timeout);
            
            loop {
                interval.tick().await;
                
                let mut operations = pending_operations.lock().await;
                if operations.is_empty() {
                    continue;
                }

                // Take operations to process
                let batch: Vec<BatchOperation> = if operations.len() >= batch_size {
                    operations.drain(..batch_size).collect()
                } else {
                    operations.drain(..).collect()
                };
                drop(operations);

                if !batch.is_empty() {
                    Self::process_batch(batch).await;
                }
            }
        });

        *handle = Some(processor_handle);
        info!("Batch processor started");
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let mut handle = self.processor_handle.lock().await;
        if let Some(processor_handle) = handle.take() {
            processor_handle.abort();
            info!("Batch processor stopped");
        }
        Ok(())
    }

    pub async fn add_operation(&self, operation: BatchOperation) -> Result<()> {
        let mut operations = self.pending_operations.lock().await;
        operations.push(operation);
        
        // If we've reached batch size, process immediately
        if operations.len() >= self.batch_size {
            let batch = operations.drain(..).collect();
            drop(operations);
            
            tokio::spawn(async move {
                Self::process_batch(batch).await;
            });
        }
        
        Ok(())
    }

    pub async fn add_items<T>(&self, items: Vec<T>) -> Result<()>
    where
        T: Into<BatchOperation> + Send + 'static,
    {
        let operations: Vec<BatchOperation> = items.into_iter().map(|item| item.into()).collect();
        
        let mut pending = self.pending_operations.lock().await;
        pending.extend(operations);
        
        Ok(())
    }

    pub async fn add_operations(&self, operations: Vec<BatchOperation>) -> Result<()> {
        let mut pending = self.pending_operations.lock().await;
        pending.extend(operations);
        Ok(())
    }

    pub async fn flush(&self) -> Result<BatchResult> {
        let mut operations = self.pending_operations.lock().await;
        if operations.is_empty() {
            return Ok(BatchResult {
                processed: 0,
                failed: 0,
                duration: Duration::from_secs(0),
            });
        }

        let batch = operations.drain(..).collect();
        drop(operations);

        Ok(Self::process_batch(batch).await)
    }

    pub async fn pending_count(&self) -> usize {
        let operations = self.pending_operations.lock().await;
        operations.len()
    }

    async fn process_batch(batch: Vec<BatchOperation>) -> BatchResult {
        let start_time = Instant::now();
        let mut processed = 0;
        let mut failed = 0;

        debug!("Processing batch of {} operations", batch.len());

        for operation in batch {
            match Self::process_operation(operation).await {
                Ok(_) => processed += 1,
                Err(e) => {
                    warn!("Failed to process operation: {}", e);
                    failed += 1;
                }
            }
        }

        let duration = start_time.elapsed();
        info!("Batch processed: {} succeeded, {} failed in {:?}", processed, failed, duration);

        BatchResult {
            processed,
            failed,
            duration,
        }
    }

    async fn process_operation(operation: BatchOperation) -> Result<()> {
        match operation {
            BatchOperation::BlockInsert { height, data } => {
                // Simulate block insertion
                debug!("Processing block insert for height {}", height);
                tokio::time::sleep(Duration::from_micros(100)).await;
            }
            
            BatchOperation::AccountUpdate { address, data } => {
                // Simulate account update
                debug!("Processing account update for {}", address);
                tokio::time::sleep(Duration::from_micros(50)).await;
            }
            
            BatchOperation::TransactionStore { tx_id, data } => {
                // Simulate transaction storage
                debug!("Processing transaction store for {}", tx_id);
                tokio::time::sleep(Duration::from_micros(75)).await;
            }
            
            BatchOperation::ContractExecution { contract_id, input } => {
                // Simulate contract execution
                debug!("Processing contract execution for {}", contract_id);
                tokio::time::sleep(Duration::from_micros(200)).await;
            }
            
            BatchOperation::NetworkMessage { peer_id, message } => {
                // Simulate network message processing
                debug!("Processing network message from {}", peer_id);
                tokio::time::sleep(Duration::from_micros(25)).await;
            }
        }
        
        Ok(())
    }
}

/// Specialized batch processors for different operation types
pub struct SpecializedBatchProcessor {
    storage_processor: BatchProcessor,
    network_processor: BatchProcessor,
    contract_processor: BatchProcessor,
}

impl SpecializedBatchProcessor {
    pub fn new() -> Self {
        Self {
            storage_processor: BatchProcessor::new(500, Duration::from_millis(50)),
            network_processor: BatchProcessor::new(1000, Duration::from_millis(10)),
            contract_processor: BatchProcessor::new(100, Duration::from_millis(100)),
        }
    }

    pub async fn start_all(&self) -> Result<()> {
        self.storage_processor.start().await?;
        self.network_processor.start().await?;
        self.contract_processor.start().await?;
        Ok(())
    }

    pub async fn stop_all(&self) -> Result<()> {
        self.storage_processor.stop().await?;
        self.network_processor.stop().await?;
        self.contract_processor.stop().await?;
        Ok(())
    }

    pub async fn add_storage_operation(&self, operation: BatchOperation) -> Result<()> {
        match operation {
            BatchOperation::BlockInsert { .. } | 
            BatchOperation::AccountUpdate { .. } | 
            BatchOperation::TransactionStore { .. } => {
                self.storage_processor.add_operation(operation).await
            }
            _ => {
                warn!("Invalid operation type for storage processor");
                Ok(())
            }
        }
    }

    pub async fn add_network_operation(&self, operation: BatchOperation) -> Result<()> {
        match operation {
            BatchOperation::NetworkMessage { .. } => {
                self.network_processor.add_operation(operation).await
            }
            _ => {
                warn!("Invalid operation type for network processor");
                Ok(())
            }
        }
    }

    pub async fn add_contract_operation(&self, operation: BatchOperation) -> Result<()> {
        match operation {
            BatchOperation::ContractExecution { .. } => {
                self.contract_processor.add_operation(operation).await
            }
            _ => {
                warn!("Invalid operation type for contract processor");
                Ok(())
            }
        }
    }

    pub async fn flush_all(&self) -> Result<(BatchResult, BatchResult, BatchResult)> {
        let storage_result = self.storage_processor.flush().await?;
        let network_result = self.network_processor.flush().await?;
        let contract_result = self.contract_processor.flush().await?;
        
        Ok((storage_result, network_result, contract_result))
    }

    pub async fn get_pending_counts(&self) -> (usize, usize, usize) {
        let storage_pending = self.storage_processor.pending_count().await;
        let network_pending = self.network_processor.pending_count().await;
        let contract_pending = self.contract_processor.pending_count().await;
        
        (storage_pending, network_pending, contract_pending)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_batch_processor_basic() {
        let processor = BatchProcessor::new(3, Duration::from_millis(100));
        
        // Add some operations
        processor.add_operation(BatchOperation::BlockInsert { 
            height: 1, 
            data: vec![1, 2, 3] 
        }).await.unwrap();
        
        processor.add_operation(BatchOperation::AccountUpdate { 
            address: "addr1".to_string(), 
            data: vec![4, 5, 6] 
        }).await.unwrap();
        
        assert_eq!(processor.pending_count().await, 2);
        
        // Flush and check results
        let result = processor.flush().await.unwrap();
        assert_eq!(result.processed, 2);
        assert_eq!(result.failed, 0);
        assert_eq!(processor.pending_count().await, 0);
    }

    #[tokio::test]
    async fn test_batch_processor_auto_flush() {
        let processor = BatchProcessor::new(2, Duration::from_millis(50));
        processor.start().await.unwrap();
        
        // Add operations that should trigger auto-flush
        processor.add_operation(BatchOperation::BlockInsert { 
            height: 1, 
            data: vec![1, 2, 3] 
        }).await.unwrap();
        
        processor.add_operation(BatchOperation::BlockInsert { 
            height: 2, 
            data: vec![4, 5, 6] 
        }).await.unwrap();
        
        // Should auto-flush when batch size is reached
        sleep(Duration::from_millis(10)).await;
        
        processor.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_specialized_batch_processor() {
        let processor = SpecializedBatchProcessor::new();
        processor.start_all().await.unwrap();
        
        processor.add_storage_operation(BatchOperation::BlockInsert { 
            height: 1, 
            data: vec![1, 2, 3] 
        }).await.unwrap();
        
        processor.add_network_operation(BatchOperation::NetworkMessage { 
            peer_id: "peer1".to_string(), 
            message: vec![4, 5, 6] 
        }).await.unwrap();
        
        let (storage_pending, network_pending, contract_pending) = processor.get_pending_counts().await;
        assert_eq!(storage_pending, 1);
        assert_eq!(network_pending, 1);
        assert_eq!(contract_pending, 0);
        
        processor.stop_all().await.unwrap();
    }
}