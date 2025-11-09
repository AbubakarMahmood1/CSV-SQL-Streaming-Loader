//! Batch processing with retry logic

use crate::errors::{LoaderError, Result};
use crate::db::CopyLoader;
use std::time::Duration;
use tokio::time::sleep;

/// Batch processor configuration
#[derive(Debug, Clone)]
pub struct BatchConfig {
    pub batch_size: usize,
    pub max_retries: usize,
    pub initial_backoff: Duration,
    pub max_backoff: Duration,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            batch_size: 10_000,
            max_retries: 3,
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(60),
        }
    }
}

/// Batch processor
pub struct BatchProcessor {
    config: BatchConfig,
}

impl BatchProcessor {
    pub fn new(config: BatchConfig) -> Self {
        Self { config }
    }

    /// Process a batch with retry logic
    pub async fn process_batch(
        &self,
        loader: &CopyLoader<'_>,
        batch: Vec<Vec<String>>,
    ) -> Result<u64> {
        let mut retries = 0;
        let mut backoff = self.config.initial_backoff;

        loop {
            match loader.load_batch(&batch).await {
                Ok(count) => return Ok(count),
                Err(e) => {
                    if retries >= self.config.max_retries {
                        return Err(LoaderError::BatchError {
                            retries,
                            message: e.to_string(),
                        });
                    }

                    tracing::warn!(
                        "Batch failed (attempt {}/{}): {}. Retrying in {:?}...",
                        retries + 1,
                        self.config.max_retries,
                        e,
                        backoff
                    );

                    sleep(backoff).await;

                    retries += 1;
                    backoff = std::cmp::min(backoff * 2, self.config.max_backoff);
                }
            }
        }
    }
}

/// Batch iterator - splits records into batches
pub struct BatchIterator<I> {
    iter: I,
    batch_size: usize,
}

impl<I> BatchIterator<I> {
    pub fn new(iter: I, batch_size: usize) -> Self {
        Self { iter, batch_size }
    }
}

impl<I> Iterator for BatchIterator<I>
where
    I: Iterator<Item = Result<Vec<String>>>,
{
    type Item = Result<Vec<Vec<String>>>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut batch = Vec::with_capacity(self.batch_size);

        for _ in 0..self.batch_size {
            match self.iter.next() {
                Some(Ok(row)) => batch.push(row),
                Some(Err(e)) => return Some(Err(e)),
                None => break,
            }
        }

        if batch.is_empty() {
            None
        } else {
            Some(Ok(batch))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_iterator() {
        let data: Vec<Result<Vec<String>>> = vec![
            Ok(vec!["1".to_string()]),
            Ok(vec!["2".to_string()]),
            Ok(vec!["3".to_string()]),
            Ok(vec!["4".to_string()]),
            Ok(vec!["5".to_string()]),
        ];

        let mut batches = BatchIterator::new(data.into_iter(), 2);

        let batch1 = batches.next().unwrap().unwrap();
        assert_eq!(batch1.len(), 2);

        let batch2 = batches.next().unwrap().unwrap();
        assert_eq!(batch2.len(), 2);

        let batch3 = batches.next().unwrap().unwrap();
        assert_eq!(batch3.len(), 1);

        assert!(batches.next().is_none());
    }

    #[test]
    fn test_default_batch_config() {
        let config = BatchConfig::default();
        assert_eq!(config.batch_size, 10_000);
        assert_eq!(config.max_retries, 3);
    }
}
