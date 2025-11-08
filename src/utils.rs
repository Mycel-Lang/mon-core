/// Calculates the 1-based line and column number for a given byte position in the source text.
/// This function is designed to be called only when an error occurs, as it iterates through
/// the source text to determine the position.
pub fn get_line_and_column(source: &str, position: usize) -> (usize, usize) {
    let mut line = 1;
    let mut column = 1;
    for (i, c) in source.chars().enumerate() {
        if i == position {
            break;
        }
        if c == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    (line, column)
}
