pub fn get_static_str(text: String) -> &'static str {
    Box::leak(text.into_boxed_str())
}

pub fn convert_text_first_char_to_uppercase(text: &str) -> String {
    let mut result = String::new();
    let mut first_char = true;
    for c in text.chars() {
        if first_char {
            result.push_str(&c.to_uppercase().to_string());
            first_char = false;
        } else {
            result.push(c);
        }
    }
    result
}
