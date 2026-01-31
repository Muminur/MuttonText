// Shared utilities

pub mod memory;

pub use memory::{PooledBuffer, clear_caches};

#[cfg(test)]
mod tests {
    #[test]
    fn test_utils_module_loads() {
        assert!(true);
    }
}
