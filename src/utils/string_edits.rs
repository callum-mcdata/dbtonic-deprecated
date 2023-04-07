pub fn remove_parentheses_content(s: &str) -> String {
    let start = s.find('(');
    let end = s.find(')');

    match (start, end) {
        (Some(start_index), Some(end_index)) => {
            let mut result = String::new();
            result.push_str(&s[..start_index]);
            result.push_str(&s[end_index + 1..]);
            result
        }
        _ => s.to_string(),
    }
}