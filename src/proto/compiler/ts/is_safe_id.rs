pub(super) fn is_safe_id(id: &str) -> bool {
    id.chars()
        .all(|c| matches!(c, '$' | '0'..='9' | 'a'..='z' | 'A'..='Z' | '_'))
}
