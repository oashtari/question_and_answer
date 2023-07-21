use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Question {
    pub id: QuestionId,
    pub title: String,
    pub content: String,
    pub tags: Option<Vec<String>>,
}
#[derive(Debug, Serialize, Eq, PartialEq, Hash, Clone, Deserialize)]
pub struct QuestionId(pub i32);
