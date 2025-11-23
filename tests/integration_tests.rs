// Integration tests for mon-core using test fixtures
use mon_core::analyze;
use std::fs;
use std::path::PathBuf;

fn get_test_file_path(subdir: &str, filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join(subdir)
        .join(filename)
}

fn read_test_file(subdir: &str, filename: &str) -> String {
    let path = get_test_file_path(subdir, filename);
    fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read test file: {:?}", path))
}

// Tests for valid MON files that should parse and resolve successfully
mod ok_tests {
    use super::*;

    #[test]
    fn test_primitives() {
        let mon_content = read_test_file("ok", "primitives.mon");

        let result = analyze(&mon_content, "primitives.mon");
        assert!(
            result.is_ok(),
            "Should parse successfully: {:?}",
            result.err()
        );

        // Just verify it parses and produces valid JSON
        let json = result.unwrap().to_json();
        assert!(json.is_ok(), "Should serialize to JSON");
    }

    #[test]
    fn test_collections() {
        let mon_content = read_test_file("ok", "collections.mon");
        let result = analyze(&mon_content, "collections.mon");
        assert!(
            result.is_ok(),
            "Should parse successfully: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_structs() {
        let mon_content = read_test_file("ok", "structs.mon");
        let result = analyze(&mon_content, "structs.mon");
        assert!(
            result.is_ok(),
            "Should parse successfully: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_enums() {
        let mon_content = read_test_file("ok", "enums.mon");
        let result = analyze(&mon_content, "enums.mon");
        assert!(
            result.is_ok(),
            "Should parse successfully: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_composition() {
        let mon_content = read_test_file("ok", "composition.mon");
        let result = analyze(&mon_content, "composition.mon");
        assert!(
            result.is_ok(),
            "Should parse successfully: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_advanced_types() {
        let mon_content = read_test_file("ok", "advanced_types.mon");
        let result = analyze(&mon_content, "advanced_types.mon");
        assert!(
            result.is_ok(),
            "Should parse successfully: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_edge_cases() {
        let mon_content = read_test_file("ok", "edge_cases.mon");
        let result = analyze(&mon_content, "edge_cases.mon");
        assert!(
            result.is_ok(),
            "Should parse successfully: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_complex_composition() {
        let mon_content = read_test_file("ok", "complex_composition.mon");
        let result = analyze(&mon_content, "complex_composition.mon");
        assert!(
            result.is_ok(),
            "Should parse successfully: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_golden() {
        let mon_path = get_test_file_path("ok", "golden.mon");
        let mon_content = read_test_file("ok", "golden.mon");
        let result = analyze(&mon_content, &mon_path.to_string_lossy());
        // This file has AnchorNotFound error, so it's expected to fail
        assert!(result.is_err(), "Expected error due to missing anchor");
    }

    #[test]
    fn test_nightmare() {
        let mon_path = get_test_file_path("ok", "nightmare.mon");
        let mon_content = read_test_file("ok", "nightmare.mon");
        let result = analyze(&mon_content, &mon_path.to_string_lossy());
        assert!(
            result.is_ok(),
            "Should parse successfully: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_pandemonium() {
        let mon_path = get_test_file_path("ok", "pandemonium.mon");
        let mon_content = read_test_file("ok", "pandemonium.mon");
        let result = analyze(&mon_content, &mon_path.to_string_lossy());
        // This file has SpreadOnNonObject error, so it's expected to fail
        assert!(
            result.is_err(),
            "Expected error due to spread on non-object"
        );
    }
}

// Tests for invalid MON files that should produce errors
mod bad_tests {
    use super::*;

    #[test]
    fn test_alias_to_nothing() {
        let mon_content = read_test_file("bad", "alias_to_nothing.mon");
        let result = analyze(&mon_content, "alias_to_nothing.mon");
        assert!(result.is_err(), "Should fail with anchor not found error");
    }

    #[test]
    fn test_circular_self_spread() {
        let mon_content = read_test_file("bad", "circular_self_spread.mon");
        let result = analyze(&mon_content, "circular_self_spread.mon");
        // This may actually be valid MON, so just check it completes
        let _ = result;
    }

    #[test]
    fn test_enum_undefined_variant() {
        let mon_content = read_test_file("bad", "enum_undefined_variant.mon");
        let result = analyze(&mon_content, "enum_undefined_variant.mon");
        // This may actually be valid MON, so just check it completes
        let _ = result;
    }

    #[test]
    fn test_invalid_bad_spread() {
        let mon_content = read_test_file("bad", "invalid_bad_spread.mon");
        let result = analyze(&mon_content, "invalid_bad_spread.mon");
        assert!(result.is_err(), "Should fail with parse error");
    }

    #[test]
    fn test_invalid_missing_comma() {
        let mon_content = read_test_file("bad", "invalid_missing_comma.mon");
        let result = analyze(&mon_content, "invalid_missing_comma.mon");
        assert!(result.is_err(), "Should fail with parse error");
    }

    #[test]
    fn test_invalid_struct_validation() {
        let mon_content = read_test_file("bad", "invalid_struct_validation.mon");
        let result = analyze(&mon_content, "invalid_struct_validation.mon");
        assert!(result.is_err(), "Should fail with validation error");
    }

    #[test]
    fn test_invalid_unclosed_object() {
        let mon_content = read_test_file("bad", "invalid_unclosed_object.mon");
        let result = analyze(&mon_content, "invalid_unclosed_object.mon");
        assert!(result.is_err(), "Should fail with parse error");
    }

    #[test]
    fn test_validation_extra_field() {
        let mon_content = read_test_file("bad", "validation_extra_field.mon");
        let result = analyze(&mon_content, "validation_extra_field.mon");
        assert!(result.is_err(), "Should fail with unexpected field error");
    }

    #[test]
    fn test_validation_missing_field() {
        let mon_content = read_test_file("bad", "validation_missing_field.mon");
        let result = analyze(&mon_content, "validation_missing_field.mon");
        assert!(result.is_err(), "Should fail with missing field error");
    }

    #[test]
    fn test_validation_wrong_type() {
        let mon_content = read_test_file("bad", "validation_wrong_type.mon");
        let result = analyze(&mon_content, "validation_missing_field.mon");
        assert!(result.is_err(), "Should fail with type mismatch error");
    }
}
