pub(crate) fn camel_to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c.is_uppercase() {
            if let Some(next) = chars.peek() {
                if next.is_lowercase() {
                    if !result.is_empty() {
                        result.push('_');
                    }
                    result.push(c.to_ascii_lowercase());
                } else {
                    result.push(c.to_ascii_lowercase());
                }
            } else {
                result.push(c.to_ascii_lowercase());
            }
        } else {
            result.push(c);
        }
    }

    result
}
