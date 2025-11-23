// Additional parser error path tests
// These systematically test unhappy paths to improve coverage

use mon_core::analyze;

#[test]
fn test_parser_error_missing_closing_brace() {
    let source = "{ key: 123";
    let result = analyze(source, "test.mon");
    assert!(result.is_err(), "Should fail with missing }}");
}

#[test]
fn test_parser_error_missing_closing_bracket() {
    let source = "{ arr: [1, 2, 3 }";
    let result = analyze(source, "test.mon");
    assert!(result.is_err(), "Should fail with missing ]");
}

#[test]
fn test_parser_error_missing_colon() {
    let source = "{ key 123 }";
    let result = analyze(source, "test.mon");
    assert!(result.is_err(), "Should fail with missing :");
}

#[test]
fn test_parser_error_unexpected_eof() {
    let source = "{ key: ";
    let result = analyze(source, "test.mon");
    assert!(result.is_err(), "Should fail with unexpected EOF");
}

#[test]
fn test_parser_error_invalid_type_spec() {
    let source = "{ value :: }";
    let result = analyze(source, "test.mon");
    assert!(result.is_err(), "Should fail with incomplete type spec");
}

#[test]
fn test_parser_error_incomplete_struct() {
    let source = "{ MyType: #struct }";
    let result = analyze(source, "test.mon");
    assert!(result.is_err(), "Should fail with incomplete struct");
}

#[test]
fn test_parser_error_incomplete_enum() {
    let source = "{ Status: #enum }";
    let result = analyze(source, "test.mon");
    assert!(result.is_err(), "Should fail with incomplete enum");
}

#[test]
fn test_parser_error_invalid_anchor() {
    let source = "{ &: value }";
    let result = analyze(source, "test.mon");
    assert!(result.is_err(), "Should fail with invalid anchor");
}

#[test]
fn test_parser_error_malformed_import() {
    let source = "import from \"file.mon\" {}";
    let result = analyze(source, "test.mon");
    assert!(result.is_err(), "Should fail with malformed import");
}

#[test]
fn test_parser_error_import_missing_from() {
    let source = "import { A } \"file.mon\" {}";
    let result = analyze(source, "test.mon");
    assert!(result.is_err(), "Should fail without 'from' keyword");
}

#[test]
fn test_parser_error_double_comma() {
    let source = "{ a: 1,, b: 2 }";
    let result = analyze(source, "test.mon");
    assert!(result.is_err(), "Should fail with double comma");
}

#[test]
fn test_parser_error_invalid_field_def() {
    let source = "{ T: #struct { field } }";
    let result = analyze(source, "test.mon");
    assert!(result.is_err(), "Should fail with invalid field");
}

#[test]
fn test_parser_error_unexpected_after_hash() {
    let source = "{ T: #invalid {} }";
    let result = analyze(source, "test.mon");
    assert!(result.is_err(), "Should fail with invalid keyword after #");
}

#[test]
fn test_parser_error_incomplete_alias() {
    let source = "{ value: * }";
    let result = analyze(source, "test.mon");
    assert!(result.is_err(), "Should fail with incomplete alias");
}

#[test]
fn test_parser_error_empty_collection_type() {
    let source = "{ value :: [] = 1 }";
    let result = analyze(source, "test.mon");
    assert!(result.is_err(), "Should fail with empty collection type");
}
