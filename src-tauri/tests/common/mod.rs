use std::path::PathBuf;
use tempfile::TempDir;
use uuid::Uuid;

/// Creates a temporary directory for testing
/// The directory is automatically cleaned up when the returned TempDir is dropped
pub fn create_temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

/// Creates a temporary directory and returns its path as a PathBuf
/// Note: The directory will be cleaned up when the TempDir is dropped,
/// so you may need to keep the TempDir alive for the duration of your test
pub fn create_temp_dir_path() -> (TempDir, PathBuf) {
    let temp_dir = create_temp_dir();
    let path = temp_dir.path().to_path_buf();
    (temp_dir, path)
}

/// Test fixture: Creates a mock combo with default values
#[cfg(test)]
pub fn create_test_combo(keyword: &str, snippet: &str) -> serde_json::Value {
    serde_json::json!({
        "id": Uuid::new_v4().to_string(),
        "keyword": keyword,
        "snippet": snippet,
        "case_sensitive": false,
        "propagate_case": false,
        "word_boundary": true,
        "group_id": null,
        "created_at": chrono::Utc::now().to_rfc3339(),
        "updated_at": chrono::Utc::now().to_rfc3339(),
    })
}

/// Test fixture: Creates a mock group with default values
#[cfg(test)]
pub fn create_test_group(name: &str) -> serde_json::Value {
    serde_json::json!({
        "id": Uuid::new_v4().to_string(),
        "name": name,
        "enabled": true,
        "created_at": chrono::Utc::now().to_rfc3339(),
        "updated_at": chrono::Utc::now().to_rfc3339(),
    })
}

/// Helper to assert that a Result is Ok and return the value
#[cfg(test)]
pub fn assert_ok<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(err) => panic!("Expected Ok, got Err: {:?}", err),
    }
}

/// Helper to assert that a Result is Err
#[cfg(test)]
pub fn assert_err<T: std::fmt::Debug, E>(result: Result<T, E>) {
    match result {
        Ok(value) => panic!("Expected Err, got Ok: {:?}", value),
        Err(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_temp_dir() {
        let temp_dir = create_temp_dir();
        assert!(temp_dir.path().exists());
    }

    #[test]
    fn test_create_temp_dir_path() {
        let (_temp_dir, path) = create_temp_dir_path();
        assert!(path.exists());
    }

    #[test]
    fn test_create_test_combo() {
        let combo = create_test_combo("test", "Test Snippet");
        assert_eq!(combo["keyword"], "test");
        assert_eq!(combo["snippet"], "Test Snippet");
        assert_eq!(combo["case_sensitive"], false);
        assert_eq!(combo["word_boundary"], true);
    }

    #[test]
    fn test_create_test_group() {
        let group = create_test_group("Test Group");
        assert_eq!(group["name"], "Test Group");
        assert_eq!(group["enabled"], true);
    }

    #[test]
    fn test_assert_ok() {
        let result: Result<i32, &str> = Ok(42);
        let value = assert_ok(result);
        assert_eq!(value, 42);
    }

    #[test]
    #[should_panic(expected = "Expected Ok, got Err")]
    fn test_assert_ok_panics_on_err() {
        let result: Result<i32, &str> = Err("error");
        assert_ok(result);
    }

    #[test]
    fn test_assert_err() {
        let result: Result<i32, &str> = Err("error");
        assert_err(result);
    }

    #[test]
    #[should_panic(expected = "Expected Err, got Ok")]
    fn test_assert_err_panics_on_ok() {
        let result: Result<i32, &str> = Ok(42);
        assert_err(result);
    }
}
