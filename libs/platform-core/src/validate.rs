#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError(pub &'static str);

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

pub fn trim_note_body(body: &str) -> Result<String, ValidationError> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return Err(ValidationError("body must not be empty"));
    }
    Ok(trimmed.to_string())
}
