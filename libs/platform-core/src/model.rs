#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    pub id: String,
    pub body: String,
    pub created_at_ms: i64,
}
