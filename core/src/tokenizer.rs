/// Minimal command tokenizer used by simulation parsing.
pub fn tokenize_command(raw: &str) -> Vec<String> {
    raw.split_whitespace().map(|s| s.to_string()).collect()
}
