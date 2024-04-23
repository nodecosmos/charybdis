pub trait ToSnakeCase {
    fn to_snake_case(&self) -> String;
}

impl ToSnakeCase for str {
    fn to_snake_case(&self) -> String {
        let mut result = String::new();
        let mut prev_char_is_uppercase = false;

        for (i, c) in self.chars().enumerate() {
            if c.is_uppercase() {
                if i != 0 && !prev_char_is_uppercase {
                    result.push('_');
                }
                result.push(c.to_ascii_lowercase());
                prev_char_is_uppercase = true;
            } else {
                result.push(c);
                prev_char_is_uppercase = false;
            }
        }

        result
    }
}
