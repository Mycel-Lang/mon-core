use mon_core::api::analyze;

#[test]
fn test_simple_parse_to_json() {
    let source = r#"
        {
            name: "My App",
            version: 1.0,
            is_enabled: true,
            features: ["a", "b", "c"],
            config: {
                host: "localhost",
                port: 8080.0,
            }
        }
    "#;

    let expected_json = serde_json::json!({
        "name": "My App",
        "version": 1.0,
        "is_enabled": true,
        "features": ["a", "b", "c"],
        "config": {
            "host": "localhost",
            "port": 8080.0,
        }
    });

    let analysis_result = analyze(source, "test.mon").unwrap();
    let result = analysis_result.to_json().unwrap();
    let result_json: serde_json::Value = serde_json::from_str(&result).unwrap();

    assert_eq!(result_json, expected_json);
}

#[test]
fn test_analyze_semantic_info() {
    let source = r#"
        {
            MyType: #struct { field(String) },
            &my_anchor: { a: 1 },
            value: *my_anchor,
        }
    "#;

    let analysis_result = analyze(source, "test.mon").unwrap();

    // Check symbol table
    assert!(analysis_result.symbol_table.types.contains_key("MyType"));

    // Check anchors
    assert!(analysis_result.anchors.contains_key("my_anchor"));
}

#[test]
fn test_simple_parse_to_yaml() {
    let source = r#"
        {
            name: "My App",
            version: 1.0,
            is_enabled: true,
        }
    "#;

    let expected_yaml = "is_enabled: true\nname: My App\nversion: 1.0\n";

    let analysis_result = analyze(source, "test.mon").unwrap();
    let result = analysis_result.to_yaml().unwrap();

    assert_eq!(result, expected_yaml);
}
