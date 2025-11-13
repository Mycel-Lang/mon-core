use miette::Report;
use mon_core::parser::Parser;
use std::fs;

#[test]
fn test_all_mon_files() {
    let tests_dir = "./tests";
    let entries = fs::read_dir(tests_dir).expect("Failed to read tests directory");

    for entry in entries {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.is_file() && path.extension().map_or(false, |ext| ext == "mon") {
            println!("Parsing file: {:?}", path);
            let source =
                fs::read_to_string(&path).expect(&format!("Failed to read file: {:?}", path));

            let mut parser = Parser::new(&source).expect("Lexer failed");

            if let Err(err) = parser.parse_document() {
                panic!("Failed to parse {:?}. Error: {:#?}", path, Report::new(err));
            }
        }
    }
}
