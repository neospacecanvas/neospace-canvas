use crate::parallel::{calculate_chunk_size, MIN_CHUNK_SIZE};
use rayon::prelude::*;

//TODO: add memory efficient batching ChunkResult<T>
//TODO: use .try_fold to process in place without adding new vectors
//TODO: share processors between threads let processor = Arc::new(processor);
//TODO: cache line alignment optimization more precise cunk sizing
//  let elements_per_cache_line = cache_line_size / elem_size

#[derive(Debug)]
pub enum ProcessingError {
    ProcessingFailed(String),
}

/// parallel execution engine
pub struct ParallelExecutor {
    chunk_size: usize,
}

impl ParallelExecutor {
    pub fn new() -> Self {
        Self {
            chunk_size: MIN_CHUNK_SIZE,
        }
    }

    /// process single column of data in parallel
    /// F is the function that processes each chunk
    /// C is the function that combines results
    /// T will be some datatype that there is an array of being processed
    /// R is the Rusult tipe if there is not some failure that throws ProcessingError
    pub fn process_column<T, R, F, C>(
        &self,
        data: &[T],
        processor: F,
        combiner: C,
    ) -> Result<R, ProcessingError>
    where
        T: Send + Sync,
        R: Send,
        F: Fn(&[T]) -> R + Send + Sync,
        C: Fn(R, R) -> R + Send + Sync,
    {
        // split the data into chunk
        let chunks: Vec<&[T]> = data.chunks(self.chunk_size).collect();
        // process chunks in parallel
        let results: Vec<R> = chunks.par_iter().map(|chunk| processor(chunk)).collect();
        // combine results
        let final_result = results
            .into_iter()
            .reduce(|a, b| combiner(a, b))
            .ok_or_else(|| ProcessingError::ProcessingFailed("No data processed".into()))?;

        Ok(final_result)
    }
    /// type, result, function
    pub fn process_columns<T, R, F, C>(
        &self,
        columns: &[Vec<T>],
        processor: F,
        combiner: C,
    ) -> Result<Vec<R>, ProcessingError>
    where
        T: Send + Sync,
        R: Send,
        F: Fn(&[T]) -> R + Send + Sync + Clone,
        C: Fn(R, R) -> R + Send + Sync + Clone,
    {
        // Process each column in parallel
        let results: Vec<R> = columns
            .par_iter()
            .map(|column| {
                // Process all chunks and combine their results
                let chunks: Vec<&[T]> = column.chunks(self.chunk_size).collect();
                chunks
                    .par_iter()
                    .map(|chunk| processor(chunk))
                    .reduce(|| processor(&[]), |a, b| combiner(a, b))
            })
            .collect();

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_column_processing() {
        let data: Vec<i32> = (0..1000).collect(); // Larger than chunk size
        let executor = ParallelExecutor::new();

        // Sum all numbers
        let processor = |chunk: &[i32]| chunk.iter().sum::<i32>();
        let combiner = |a, b| a + b;

        let result = executor.process_column(&data, processor, combiner).unwrap();
        let expected: i32 = data.iter().sum();
        assert_eq!(result, expected, "Failed to process all chunks in column");
    }

    #[test]
    fn test_multi_column_processing() {
        // Create multiple columns larger than chunk_size
        let columns = vec![
            (0..1000).collect::<Vec<i32>>(),    // Column 1: 0..999
            (1000..2000).collect::<Vec<i32>>(), // Column 2: 1000..1999
        ];
        let executor = ParallelExecutor::new();

        // Process columns to get their sums
        let processor = |chunk: &[i32]| chunk.iter().sum::<i32>();
        let combiner = |a, b| a + b;

        let results = executor
            .process_columns(&columns, processor, combiner)
            .unwrap();

        // Verify results
        assert_eq!(results.len(), 2, "Should have result for each column");
        assert_eq!(
            results[0],
            (0..1000).sum::<i32>(),
            "First column sum incorrect"
        );
        assert_eq!(
            results[1],
            (1000..2000).sum::<i32>(),
            "Second column sum incorrect"
        );
    }

    #[test]
    fn test_empty_and_small_columns() {
        let columns = vec![
            vec![],              // Empty column
            vec![1],             // Single element
            vec![1, 2, 3],       // Small column
            (0..1000).collect(), // Large column
        ];
        let executor = ParallelExecutor::new();

        // Count elements in each chunk
        let processor = |chunk: &[i32]| chunk.len();
        let combiner = |a, b| a + b;

        let results = executor
            .process_columns(&columns, processor, combiner)
            .unwrap();

        assert_eq!(results[0], 0, "Empty column should have 0 elements");
        assert_eq!(results[1], 1, "Single element column");
        assert_eq!(results[2], 3, "Small column");
        assert_eq!(results[3], 1000, "Large column");
    }

    #[test]
    fn test_uneven_columns() {
        let columns = vec![
            vec![1; 1000], // 1000 ones
            vec![2; 750],  // 750 twos
            vec![3; 1250], // 1250 threes
        ];
        let executor = ParallelExecutor::new();

        // Sum each column
        let processor = |chunk: &[i32]| chunk.iter().sum::<i32>();
        let combiner = |a, b| a + b;

        let results = executor
            .process_columns(&columns, processor, combiner)
            .unwrap();

        assert_eq!(results[0], 1000, "First column sum");
        assert_eq!(results[1], 1500, "Second column sum");
        assert_eq!(results[2], 3750, "Third column sum");
    }

    #[test]
    fn test_chunk_boundaries() {
        // Create a column exactly 2.5 times the chunk size
        let executor = ParallelExecutor::new();
        let chunk_size = executor.chunk_size;
        let test_size = chunk_size * 2 + chunk_size / 2;

        let data: Vec<i32> = (0..test_size as i32).collect();

        // Sum all numbers
        let processor = |chunk: &[i32]| chunk.iter().sum::<i32>();
        let combiner = |a, b| a + b;

        let result = executor.process_column(&data, processor, combiner).unwrap();
        let expected: i32 = data.iter().sum();

        assert_eq!(
            result, expected,
            "Failed to process column with partial chunks correctly"
        );
    }
}
