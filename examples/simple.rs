use mon_core::analyze;

fn main() {
    let mon_data = r#"
        user: {
            name: "John Doe",
            email: "john.doe@example.com"
        }
    "#;

    match analyze(mon_data, "example.mon") {
        Ok(result) => {
            let json_output = result.to_json().unwrap();
            println!("Successfully parsed MON to JSON:\n{json_output}");
        }
        Err(e) => {
            eprintln!("Failed to parse MON: {e:?}");
        }
    }
}

