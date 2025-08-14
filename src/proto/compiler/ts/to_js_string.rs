pub(super) fn to_js_string(text: &str) -> String {
    let mut res = String::new();
    res.push('"');
    for char in text.chars() {
        match char {
            '\"' => res.push_str(r#"\""#),
            '\\' => res.push_str(r"\\"),
            _ => res.push(char),
        }

    }
    res.push('"');
    res
}