use std::collections::HashMap;
// use std::sync::Arc;
// use tokio::sync::RwLock;
use sqlx::postgres::{PgPool, PgPoolOptions, PgRow};
use sqlx::Row;

use crate::types::{
    answer::{Answer, AnswerId},
    question::{Question, QuestionId},
};

#[derive(Debug, Clone)]
pub struct Store {
    // PRE SQLX
    // pub questions: Arc<RwLock<HashMap<QuestionId, Question>>>,
    // pub answers: Arc<RwLock<HashMap<AnswerId, Answer>>>,
    pub connection: PgPool,
}

impl Store {
    pub async fn new(db_url: &str) -> Self {
        let db_pool = match PgPoolOptions::new()
            .max_connections(5)
            .connect(db_url)
            .await
        {
            Ok(pool) => pool,
            Err(e) => panic!("Couldn't establish DB connection:[]", e),
        };
        Store {
            connection: db_pool,
            // PRE SQLX
            // questions: Arc::new(RwLock::new(Self::init())),
            // answers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // fn add_question(mut self, question: Question) -> Self {
    //     self.questions.insert(question.id.clone(), question);
    //     self
    // }

    // fn init(self) -> Self {
    //     let question = Question::new(
    //         QuestionId::from_str("1").expect("Id not set"),
    //         "How?".to_string(),
    //         "Please help!".to_string(),
    //         Some(vec!["general".to_string()]),
    //     );
    //     self.add_question(question)
    // }

    fn init() -> HashMap<QuestionId, Question> {
        let file = include_str!("../questions.json");
        serde_json::from_str(file).expect("can't read questions.json")
    }
}
