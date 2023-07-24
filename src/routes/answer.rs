use std::collections::HashMap;
use warp::http::StatusCode;

use crate::store::Store;
use crate::types::{
    answer::{Answer, AnswerId, NewAnswer},
    question::QuestionId,
};

pub async fn add_answer(
    store: Store,
    new_answer: NewAnswer,
    // params: HashMap<String, String>,
) -> Result<impl warp::Reply, warp::Rejection> {
    match store.add_answer(new_answer).await {
        Ok(_) => Ok(warp::reply::with_status("Answer added", StatusCode::OK)),
        Err(e) => Err(warp::reject::custom(e)),
    }

    // let answer = Answer {
    //     id: AnswerId("1".to_string()),
    //     content: params.get("content").unwrap().to_string(),
    //     question_id: QuestionId(params.get("questionId").unwrap().to_string()),
    // };

    // store
    //     .answers
    //     .write()
    //     .await
    //     .insert(answer.id.clone(), answer);

    // Ok(warp::reply::with_status("Answer added", StatusCode::OK))
}
