//! MT-1105: Memory management utilities for MuttonText.
//!
//! Provides `PooledBuffer<T>` for reusing Vec allocations and a `clear_caches()`
//! function that managers can call to release unused memory.

use std::cell::RefCell;

/// A thread-local pool of reusable `Vec<T>` buffers.
///
/// Instead of allocating new Vecs for temporary work, callers can acquire
/// a pre-allocated buffer from the pool and return it when done.
pub struct PooledBuffer<T> {
    pool: RefCell<Vec<Vec<T>>>,
    max_pool_size: usize,
}

impl<T> PooledBuffer<T> {
    /// Creates a new buffer pool that retains up to `max_pool_size` buffers.
    pub fn new(max_pool_size: usize) -> Self {
        Self {
            pool: RefCell::new(Vec::new()),
            max_pool_size,
        }
    }

    /// Acquires a buffer from the pool, or creates a new one if the pool is empty.
    /// The returned buffer is cleared (length 0) but retains its allocation.
    pub fn acquire(&self) -> Vec<T> {
        let mut pool = self.pool.borrow_mut();
        match pool.pop() {
            Some(mut buf) => {
                buf.clear();
                buf
            }
            None => Vec::new(),
        }
    }

    /// Returns a buffer to the pool for reuse.
    /// If the pool is full, the buffer is dropped.
    pub fn release(&self, buf: Vec<T>) {
        let mut pool = self.pool.borrow_mut();
        if pool.len() < self.max_pool_size {
            pool.push(buf);
        }
        // else: buffer is dropped, freeing memory
    }

    /// Returns the number of buffers currently in the pool.
    pub fn pool_size(&self) -> usize {
        self.pool.borrow().len()
    }

    /// Clears all pooled buffers, freeing their memory.
    pub fn clear(&self) {
        self.pool.borrow_mut().clear();
    }
}

impl<T> Default for PooledBuffer<T> {
    fn default() -> Self {
        Self::new(8)
    }
}

/// Releases unused memory across all caches.
///
/// This is a centralized function that managers can call periodically
/// (e.g., after large operations or when the system is idle) to free
/// memory that is no longer needed.
///
/// Currently a no-op placeholder; individual managers should call their
/// own `compact()` or `clear_caches()` methods as they are implemented.
pub fn clear_caches() {
    tracing::debug!("clear_caches: releasing unused memory");
    // Future: call into each manager's cache clearing method
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pooled_buffer_acquire_empty_pool() {
        let pool: PooledBuffer<u8> = PooledBuffer::new(4);
        let buf = pool.acquire();
        assert!(buf.is_empty());
        assert_eq!(buf.capacity(), 0);
    }

    #[test]
    fn test_pooled_buffer_release_and_reacquire() {
        let pool: PooledBuffer<u8> = PooledBuffer::new(4);
        let mut buf = pool.acquire();
        buf.extend_from_slice(&[1, 2, 3, 4, 5]);
        let cap = buf.capacity();
        pool.release(buf);

        assert_eq!(pool.pool_size(), 1);

        let reacquired = pool.acquire();
        assert!(reacquired.is_empty()); // cleared
        assert!(reacquired.capacity() >= cap); // retains allocation
        assert_eq!(pool.pool_size(), 0);
    }

    #[test]
    fn test_pooled_buffer_max_pool_size() {
        let pool: PooledBuffer<u8> = PooledBuffer::new(2);
        pool.release(Vec::new());
        pool.release(Vec::new());
        pool.release(Vec::new()); // should be dropped, pool is full
        assert_eq!(pool.pool_size(), 2);
    }

    #[test]
    fn test_pooled_buffer_clear() {
        let pool: PooledBuffer<u8> = PooledBuffer::new(4);
        pool.release(Vec::new());
        pool.release(Vec::new());
        assert_eq!(pool.pool_size(), 2);
        pool.clear();
        assert_eq!(pool.pool_size(), 0);
    }

    #[test]
    fn test_pooled_buffer_default() {
        let pool: PooledBuffer<String> = PooledBuffer::default();
        assert_eq!(pool.pool_size(), 0);
        // default max is 8
        for _ in 0..10 {
            pool.release(Vec::new());
        }
        assert_eq!(pool.pool_size(), 8);
    }

    #[test]
    fn test_clear_caches_does_not_panic() {
        clear_caches();
    }

    #[test]
    fn test_pooled_buffer_with_strings() {
        let pool: PooledBuffer<String> = PooledBuffer::new(4);
        let mut buf = pool.acquire();
        buf.push("hello".to_string());
        buf.push("world".to_string());
        pool.release(buf);

        let reacquired = pool.acquire();
        assert!(reacquired.is_empty());
    }

    #[test]
    fn test_pooled_buffer_multiple_acquire_release_cycles() {
        let pool: PooledBuffer<i32> = PooledBuffer::new(4);
        for i in 0..100 {
            let mut buf = pool.acquire();
            buf.push(i);
            pool.release(buf);
        }
        assert_eq!(pool.pool_size(), 1);
    }
}
