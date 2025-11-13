use mon_core::api::analyze;

#[test]
fn test_get_definition_of_alias() {
    let source = r#"
        {
            &my_anchor: { a: 1 },
            value: *my_anchor,
        }
    "#;

    let analysis_result = analyze(source, "test.mon").unwrap();

    // Find the position of "*my_anchor"
    let alias_pos = source.find("*my_anchor").unwrap();

    let definition_span = analysis_result.get_definition_at(alias_pos).unwrap();

    // Find the position of "&my_anchor"
    let anchor_def_value_pos = source.find("{ a: 1 }").unwrap();
    let anchor_def_value_end_pos = anchor_def_value_pos + "{ a: 1 }".len();

    assert_eq!(definition_span.offset(), anchor_def_value_pos);
    assert_eq!(
        definition_span.len(),
        anchor_def_value_end_pos - anchor_def_value_pos
    );
}

#[test]
fn test_get_definition_of_type() {
    let source = r#"
        {
            MyType: #struct { field(String) },
            value :: MyType = { field: "hello" },
        }
    "#;

    let analysis_result = analyze(source, "test.mon").unwrap();

    // Find the position of "MyType" in the validation
    let type_pos = source.rfind("MyType").unwrap();

    let definition_span = analysis_result.get_definition_at(type_pos).unwrap();

    // Find the position of the struct definition
    let struct_def_pos = source.find("#struct { field(String) }").unwrap();
    let struct_def_end_pos = struct_def_pos + "#struct { field(String) }".len();

    assert_eq!(definition_span.offset(), struct_def_pos);
    assert_eq!(definition_span.len(), struct_def_end_pos - struct_def_pos);
}
#[test]
fn test_get_type_info() {
    let source = r#"
        {
            MyType: #struct { field(String) },
            value :: MyType = { field: "hello" },
        }
    "#;

    let analysis_result = analyze(source, "test.mon").unwrap();

    // Find the position of "hello"
    let value_pos = source.rfind("\"hello\"").unwrap();

    let type_info = analysis_result.get_type_info_at(value_pos).unwrap();

    assert_eq!(type_info, "MyType");
}

#[test]
fn test_find_references() {
    let source = r#"
        {
            &my_anchor: { a: 1 },
            value1: *my_anchor,
            value2: *my_anchor,
        }
    "#;

    let analysis_result = analyze(source, "test.mon").unwrap();

    // Find the position of the first "*my_anchor"
    let alias_pos = source.find("*my_anchor").unwrap();

    let references = analysis_result.find_references(alias_pos).unwrap();

    assert_eq!(references.len(), 2);

    let first_ref_pos = source.find("*my_anchor").unwrap();
    let second_ref_pos = source.rfind("*my_anchor").unwrap();

    assert_eq!(references[0].offset(), first_ref_pos);
    assert_eq!(references[1].offset(), second_ref_pos);
}

#[test]
fn test_find_type_references() {
    let source = r#"
        {
            MyType: #struct { field(String) },
            value1 :: MyType = { field: "a" },
            value2 :: MyType = { field: "b" },
        }
    "#;

    let analysis_result = analyze(source, "test.mon").unwrap();

    // Find the position of the last "MyType"
    let type_pos = source.rfind("MyType").unwrap();

    let references = analysis_result.find_references(type_pos).unwrap();

    assert_eq!(references.len(), 2);

    let first_usage_pos = source.find("value1 :: MyType").unwrap() + "value1 :: ".len();
    let second_usage_pos = source.find("value2 :: MyType").unwrap() + "value2 :: ".len();

    assert_eq!(references[0].offset(), first_usage_pos);
    assert_eq!(references[1].offset(), second_usage_pos);
}
