use miette::Report;
use mon_core::parser::Parser;
use mon_core::resolver::Resolver;
use std::path::PathBuf;
use std::fs;

fn resolve_ok(source: &str, file_name: &str) -> mon_core::ast::MonDocument {
    let mut parser = Parser::new(source).unwrap();
    let document = parser.parse_document().unwrap();
    let mut resolver = Resolver::new();
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(file_name);
    match resolver.resolve(document, source, path, None) {
        Ok(doc) => doc,
        Err(err) => {
            let report = Report::from(err);
            panic!("{:#}", report);
        }
    }
}

fn resolve_err(source: &str, file_name: &str) -> mon_core::error::ResolverError {
    let mut parser = Parser::new(source).unwrap();
    let document = parser.parse_document().unwrap();
    let mut resolver = Resolver::new();
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(file_name);
    match resolver.resolve(document, source, path, None) {
        Ok(_) => panic!("Expected a ResolverError, but got Ok"),
        Err(err) => err,
    }
}

#[test]
fn test_simple_alias_resolution() {
    let source = r#"{ &my_value: 123, alias_value: *my_value }"#;
    let doc = resolve_ok(source, "test.mon");

    let root_object = match doc.root.kind {
        mon_core::ast::MonValueKind::Object(members) => members,
        _ => panic!("Expected an object"),
    };

    // Check that the alias_value is resolved to 123
    let alias_member = root_object.iter().find(|m| {
        if let mon_core::ast::Member::Pair(p) = m {
            p.key == "alias_value"
        } else {
            false
        }
    }).unwrap();

    if let mon_core::ast::Member::Pair(p) = alias_member {
        assert_eq!(p.value.kind, mon_core::ast::MonValueKind::Number(123.0));
    } else {
        panic!("Expected a pair member");
    }
}

#[test]
fn test_object_spread_resolution() {
    let source = r#"{
        &base_config: { host: "localhost", port: 8080 },
        app_config: {
            ...*base_config,
            port: 9000, // Override
            debug: true,
        }
    }"#;
    let doc = resolve_ok(source, "test.mon");

    let root_object = match doc.root.kind {
        mon_core::ast::MonValueKind::Object(members) => members,
        _ => panic!("Expected an object"),
    };

    let app_config_member = root_object.iter().find(|m| {
        if let mon_core::ast::Member::Pair(p) = m {
            p.key == "app_config"
        } else {
            false
        }
    }).unwrap();

    if let mon_core::ast::Member::Pair(p) = app_config_member {
        let app_config_object = match &p.value.kind {
            mon_core::ast::MonValueKind::Object(members) => members,
            _ => panic!("Expected app_config to be an object"),
        };

        // Check host
        let host_member = app_config_object.iter().find(|m| {
            if let mon_core::ast::Member::Pair(p) = m {
                p.key == "host"
            } else {
                false
            }
        }).unwrap();
        if let mon_core::ast::Member::Pair(p) = host_member {
            assert_eq!(p.value.kind, mon_core::ast::MonValueKind::String("localhost".to_string()));
        } else {
            panic!("Expected host to be a pair");
        }

        // Check port (overridden)
        let port_member = app_config_object.iter().find(|m| {
            if let mon_core::ast::Member::Pair(p) = m {
                p.key == "port"
            } else {
                false
            }
        }).unwrap();
        if let mon_core::ast::Member::Pair(p) = port_member {
            assert_eq!(p.value.kind, mon_core::ast::MonValueKind::Number(9000.0));
        } else {
            panic!("Expected port to be a pair");
        }

        // Check debug (new field)
        let debug_member = app_config_object.iter().find(|m| {
            if let mon_core::ast::Member::Pair(p) = m {
                p.key == "debug"
            } else {
                false
            }
        }).unwrap();
        if let mon_core::ast::Member::Pair(p) = debug_member {
            assert_eq!(p.value.kind, mon_core::ast::MonValueKind::Boolean(true));
        } else {
            panic!("Expected debug to be a pair");
        }
    } else {
        panic!("Expected app_config to be a pair member");
    }
}

#[test]
fn test_array_spread_resolution() {
    let source = r#"{
        &base_tags: ["tag1", "tag2"],
        item_tags: [
            "start",
            ...*base_tags,
            "end",
        ]
    }"#;
    let doc = resolve_ok(source, "test.mon");

    let root_object = match doc.root.kind {
        mon_core::ast::MonValueKind::Object(members) => members,
        _ => panic!("Expected an object"),
    };

    let item_tags_member = root_object.iter().find(|m| {
        if let mon_core::ast::Member::Pair(p) = m {
            p.key == "item_tags"
        } else {
            false
        }
    }).unwrap();

    if let mon_core::ast::Member::Pair(p) = item_tags_member {
        let item_tags_array = match &p.value.kind {
            mon_core::ast::MonValueKind::Array(elements) => elements,
            _ => panic!("Expected item_tags to be an array"),
        };

        assert_eq!(item_tags_array.len(), 4);
        assert_eq!(item_tags_array[0].kind, mon_core::ast::MonValueKind::String("start".to_string()));
        assert_eq!(item_tags_array[1].kind, mon_core::ast::MonValueKind::String("tag1".to_string()));
        assert_eq!(item_tags_array[2].kind, mon_core::ast::MonValueKind::String("tag2".to_string()));
        assert_eq!(item_tags_array[3].kind, mon_core::ast::MonValueKind::String("end".to_string()));
    } else {
        panic!("Expected item_tags to be a pair member");
    }
}

#[test]
fn test_struct_validation_with_defaults_and_collections_ok() {
    let source = r#"
        {
            User: #struct {
                id(Number),
                name(String),
                email(String) = "default@example.com",
                is_active(Boolean) = true,
                roles([String...]),
                permissions([String, Number]),
                log_data([String, Any...]),
                status_history([Boolean..., String]),
            },

            // Valid user with defaults
            user1 :: User = {
                id: 1,
                name: "Alice",
                roles: ["admin", "editor"],
                permissions: ["read", 1],
                log_data: ["login", { timestamp: "...", ip: "..." }],
                status_history: [true, false, "active"],
            },

            // Valid user, omitting optional fields
            user2 :: User = {
                id: 2,
                name: "Bob",
                roles: [],
                permissions: ["write", 2],
                log_data: ["logout"],
                status_history: ["inactive"],
            },
        }
    "#;

    // Test valid user1
    let doc = resolve_ok(source, "test_validation.mon");
    let root_object = match doc.root.kind {
        mon_core::ast::MonValueKind::Object(members) => members,
        _ => panic!("Expected an object"),
    };

    let user1_member = root_object.iter().find(|m| {
        if let mon_core::ast::Member::Pair(p) = m {
            p.key == "user1"
        } else {
            false
        }
    }).unwrap();

    if let mon_core::ast::Member::Pair(p) = user1_member {
        let user1_object = match &p.value.kind {
            mon_core::ast::MonValueKind::Object(members) => members,
            _ => panic!("Expected user1 to be an object"),
        };

        // Check default email
        let email_member = user1_object.iter().find(|m| {
            if let mon_core::ast::Member::Pair(p) = m {
                p.key == "email"
            } else {
                false
            }
        }).unwrap();
        if let mon_core::ast::Member::Pair(p) = email_member {
            assert_eq!(p.value.kind, mon_core::ast::MonValueKind::String("default@example.com".to_string()));
        } else {
            panic!("Expected email to be a pair");
        }

        // Check default is_active
        let is_active_member = user1_object.iter().find(|m| {
            if let mon_core::ast::Member::Pair(p) = m {
                p.key == "is_active"
            } else {
                false
            }
        }).unwrap();
        if let mon_core::ast::Member::Pair(p) = is_active_member {
            assert_eq!(p.value.kind, mon_core::ast::MonValueKind::Boolean(true));
        } else {
            panic!("Expected is_active to be a pair");
        }
    } else {
        panic!("Expected user1 to be a pair member");
    }
}

#[test]
fn test_struct_validation_missing_required_field() {
    let source = r#"
        {
            User: #struct { id(Number), name(String) },
            invalid_user :: User = { id: 3 },
        }
    "#;
    let err = resolve_err(source, "test_validation.mon");
    match err {
        mon_core::error::ResolverError::Validation(mon_core::error::ValidationError::MissingField { field_name, .. }) => {
            assert_eq!(field_name, "name");
        },
        _ => panic!("Expected MissingField error, but got {:?}", err),
    }
}

#[test]
fn test_struct_validation_wrong_id_type() {
    let source = r#"
        {
            User: #struct { id(Number), name(String) },
            invalid_user :: User = { id: "four", name: "Charlie" },
        }
    "#;
    let err = resolve_err(source, "test_validation.mon");
    match err {
        mon_core::error::ResolverError::Validation(mon_core::error::ValidationError::TypeMismatch { field_name, expected_type, found_type, .. }) => {
            assert_eq!(field_name, "id");
            assert_eq!(expected_type, "Number");
            assert!(found_type.contains("String"));
        },
        _ => panic!("Expected TypeMismatch error for id, but got {:?}", err),
    }
}

#[test]
fn test_struct_validation_unexpected_field() {
    let source = r#"
        {
            User: #struct { id(Number), name(String) },
            invalid_user :: User = { id: 5, name: "David", age: 30 },
        }
    "#;
    let err = resolve_err(source, "test_validation.mon");
    match err {
        mon_core::error::ResolverError::Validation(mon_core::error::ValidationError::UnexpectedField { field_name, .. }) => {
            assert_eq!(field_name, "age");
        },
        _ => panic!("Expected UnexpectedField error, but got {:?}", err),
    }
}

#[test]
fn test_struct_validation_roles_type_mismatch() {
    let source = r#"
        {
            User: #struct { roles([String...]) },
            invalid_user :: User = { roles: ["viewer", 123] },
        }
    "#;
    let err = resolve_err(source, "test_validation.mon");
    match err {
        mon_core::error::ResolverError::Validation(mon_core::error::ValidationError::TypeMismatch { field_name, expected_type, found_type, .. }) => {
            assert_eq!(field_name, "roles");
            assert_eq!(expected_type, "String");
            assert!(found_type.contains("Number"));
        },
        _ => panic!("Expected TypeMismatch error for roles, but got {:?}", err),
    }
}

#[test]
fn test_struct_validation_permissions_length_mismatch() {
    let source = r#"
        {
            User: #struct { permissions([String, Number]) },
            invalid_user :: User = { permissions: ["read"] },
        }
    "#;
    let err = resolve_err(source, "test_validation.mon");
    match err {
        mon_core::error::ResolverError::Validation(mon_core::error::ValidationError::TypeMismatch { field_name, expected_type, found_type, .. }) => {
            assert_eq!(field_name, "permissions");
            assert!(expected_type.contains("tuple with 2 elements"));
            assert!(found_type.contains("tuple with 1 elements"));
        },
        _ => panic!("Expected TypeMismatch error for permissions length, but got {:?}", err),
    }
}

#[test]
fn test_struct_validation_permissions_type_mismatch() {
    let source = r#"
        {
            User: #struct { permissions([String, Number]) },
            invalid_user :: User = { permissions: [8, "write"] },
        }
    "#;
    let err = resolve_err(source, "test_validation.mon");
    match err {
        mon_core::error::ResolverError::Validation(mon_core::error::ValidationError::TypeMismatch { field_name, expected_type, found_type, .. }) => {
            assert_eq!(field_name, "permissions");
            assert_eq!(expected_type, "String");
            assert!(found_type.contains("Number"));
        },
        _ => panic!("Expected TypeMismatch error for permissions types, but got {:?}", err),
    }
}

#[test]
fn test_struct_validation_log_data_first_type_mismatch() {
    let source = r#"
        {
            User: #struct { log_data([String, Any...]) },
            invalid_user :: User = { log_data: [123, "event"] },
        }
    "#;
    let err = resolve_err(source, "test_validation.mon");
    match err {
        mon_core::error::ResolverError::Validation(mon_core::error::ValidationError::TypeMismatch { field_name, expected_type, found_type, .. }) => {
            assert_eq!(field_name, "log_data");
            assert_eq!(expected_type, "String");
            assert!(found_type.contains("Number"));
        },
        _ => panic!("Expected TypeMismatch error for log_data first type, but got {:?}", err),
    }
}

#[test]
fn test_struct_validation_status_history_last_type_mismatch() {
    let source = r#"
        {
            User: #struct { status_history([Boolean..., String]) },
            invalid_user :: User = { status_history: [true, 123] },
        }
    "#;
    let err = resolve_err(source, "test_validation.mon");
    match err {
        mon_core::error::ResolverError::Validation(mon_core::error::ValidationError::TypeMismatch { field_name, expected_type, found_type, .. }) => {
            assert_eq!(field_name, "status_history");
            assert_eq!(expected_type, "String");
            assert!(found_type.contains("Number"));
        },
        _ => panic!("Expected TypeMismatch error for status_history last type, but got {:?}", err),
    }
}

#[test]
fn test_nested_struct_validation_ok() {
    let source = r#"
        {
            Profile: #struct {
                username(String),
                email(String),
            },
            User: #struct {
                id(Number),
                profile(Profile),
            },

            // Valid nested struct
            user1 :: User = {
                id: 1,
                profile: {
                    username: "alice",
                    email: "alice@example.com",
                },
            },
        }
    "#;

    resolve_ok(source, "test_nested_ok.mon");
}

#[test]
fn test_nested_struct_validation_err() {
    let source = r#"
        {
            Profile: #struct {
                username(String),
                email(String),
            },
            User: #struct {
                id(Number),
                profile(Profile),
            },

            // Invalid: Nested struct has wrong type for username
            user2 :: User = {
                id: 2,
                profile: {
                    username: 123,
                    email: "bob@example.com",
                },
            },
        }
    "#;

    let err = resolve_err(source, "test_nested_err.mon");
    match err {
        mon_core::error::ResolverError::Validation(mon_core::error::ValidationError::TypeMismatch { field_name, expected_type, found_type, .. }) => {
            assert_eq!(field_name, "username");
            assert_eq!(expected_type, "String");
            assert!(found_type.contains("Number"));
        },
        _ => panic!("Expected TypeMismatch error for username, but got {:?}", err),
    }
}

#[test]
fn test_cross_file_validation() {
    let source = fs::read_to_string("tests/cross_file_main.mon").unwrap();
    resolve_ok(&source, "tests/cross_file_main.mon");
}

#[test]
fn test_parser_for_schemas_file() {
    let source = fs::read_to_string("tests/cross_file_schemas.mon").unwrap();
    let mut parser = Parser::new(&source).unwrap();
    let _ = parser.parse_document().unwrap();
}
