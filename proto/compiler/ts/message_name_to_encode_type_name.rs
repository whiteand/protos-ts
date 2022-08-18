pub(super) fn message_name_to_encode_type_name(message_name: &str) -> String {
    format!("{}EncodeInput", message_name)
}
