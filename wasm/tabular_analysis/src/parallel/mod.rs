mod executor;
mod type_detection;
mod web_executor;

// Re-export the main components that other modules will use
pub use executor::{ChunkResult, ParallelExecutor, ProcessingError};
//pub use type_detection::{detect_column_types, TypeDetectionProcessor};
//pub use web_executor::{WebExecutor, WorkerMessage, WorkerPool};

// Constants shared across parallel processing
pub const MIN_CHUNK_SIZE: usize = 1024; // Minimum chunk size aligned with common CPU cache sizes
pub const MAX_CHUNKS_PER_THREAD: usize = 4; // Maximum chunks to avoid thread overhead
pub const OPTIMAL_CHUNK_SIZE: usize = 4096; // Default optimal chunk size for most operations

pub type ParallelResult<T> = Rusult<T, ProcessingError>;

#[inline]
pub(crate) fn calculate_chunk_size(data_len: usize, element_size: usize) -> usize {
    const CAVHE_LINE_SIZE: usize = 64;

    let elements_per_cache_line = CAVHE_LINE_SIZE / element_size;
    let optimal_elements = elements_per_cache_line * MAX_CHUNKS_PER_THREAD;

    optimal_elements.max(MIN_CHUNK_SIZE).min(data_len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_size_calculation() {
        // Test with different element sizes
        assert!(calculate_chunk_size(10000, 8) >= MIN_CHUNK_SIZE);
        assert!(calculate_chunk_size(10000, 16) >= MIN_CHUNK_SIZE);

        // Test with small data set
        let small_data_size = MIN_CHUNK_SIZE / 2;
        assert_eq!(calculate_chunk_size(small_data_size, 8), small_data_size);

        // Test with large element size
        assert!(calculate_chunk_size(10000, 128) >= MIN_CHUNK_SIZE);
    }
}
