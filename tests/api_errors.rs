// API error path tests
// These test error handling, conversions, and edge cases in the API layer

use mon_core::{analyze, error::MonError};

#[test]
fn test_api_analyze_parse_error() {
    let source = "{ invalid syntax";
    let result = analyze(source, "test.mon");
    assert!(result.is_err());
    if let Err(MonError::Parser(_)) = result {
        // Success
    } else {
        panic!("Expected parser error");
    }
}

#[test]
fn test_api_analyze_resolver_error() {
    let source = "{ value: *missing_anchor }";
    let result = analyze(source, "test.mon");
    assert!(result.is_err());
    if let Err(MonError::Resolver(_)) = result {
        // Success
    } else {
        panic!("Expected resolver error");
    }
}

#[test]
fn test_api_relative_path_handling() {
    let source = "{}";
    let result = analyze(source, "relative/path/test.mon");
    // Should succeed and normalize path
    assert!(result.is_ok());
}

#[test]
fn test_api_absolute_path_handling() {
    let source = "{}";
    let result = analyze(source, "/absolute/path/test.mon");
    // Should succeed with absolute path
    assert!(result.is_ok());
}

#[test]
fn test_api_empty_filename() {
    let source = "{}";
    let result = analyze(source, "");
    // Should still work with empty filename
    assert!(result.is_ok());
}

#[test]
fn test_api_special_chars_in_filename() {
    let source = "{}";
    let result = analyze(source, "test-file_v2.mon");
    assert!(result.is_ok());
}

#[test]
fn test_api_to_json_success() {
    let source = r#"{ key: "value", num: 42 }"#;
    let result = analyze(source, "test.mon").unwrap();
    let json = result.to_json();
    assert!(json.is_ok());
    assert!(json.unwrap().contains("key"));
}

#[test]
fn test_api_to_yaml_success() {
    let source = r#"{ key: "value", num: 42 }"#;
    let result = analyze(source, "test.mon").unwrap();
    let yaml = result.to_yaml();
    assert!(yaml.is_ok());
    assert!(yaml.unwrap().contains("key"));
}

#[test]
fn test_api_validation_error_type() {
    let source = r#"{
        T: #struct { value(Number) },
        v :: T = { value: "string" }
    }"#;
    let result = analyze(source, "test.mon");
    assert!(result.is_err());
    // Should be a resolver error with validation
    if let Err(MonError::Resolver(_)) = result {
        // Success
    } else {
        panic!("Expected resolver error");
    }
}

#[test]
fn test_api_error_display() {
    let source = "{ invalid";
    if let Err(err) = analyze(source, "test.mon") {
        let error_string = format!("{}", err);
        assert!(!error_string.is_empty());
    } else {
        panic!("Should have errored");
    }
}
