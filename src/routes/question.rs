#![warn(clippy::all)]

use std::collections::HashMap;

use tracing::{event, info, instrument, Level};
use warp::http::StatusCode;

use crate::store::Store;
use crate::types::pagination::{self, extract_pagination, Pagination};
use crate::types::question::{NewQuestion, Question, QuestionId};
// use handle_errors::Error;

#[instrument]
pub async fn get_questions(
    params: HashMap<String, String>,
    store: Store,
    // id: String, LOGGING
) -> Result<impl warp::Reply, warp::Rejection> {
    // let question = Question::new(
    //     QuestionId::from_str("1").expect("No id provided"),
    //     "First Question".to_string(),
    //     "Content of question".to_string(),
    //     Some(vec!["FAQ".to_string()]),
    // );

    // match question.id.0.parse::<i32>() {
    //     Err(_) => Err(warp::reject::custom(InvalidId)),
    //     Ok(_) => Ok(warp::reply::json(&question)),
    // }

    // USING MATCH
    // match params.get("start") {
    //     Some(start) => println!("{}", start),
    //     None => println!("No start value"),
    // }

    // CODE BEFORE PULLING OUT LOGIC INTO SEPARATE FUNCITON
    // let mut start = 0;

    // if let Some(n) = params.get("start") {
    //     start = n.parse::<usize>().expect("Could not parse start.")
    // }
    // println!("{}", start);

    // log::info!("Start querying questions."); LOGGING
    info!("querying questions"); // TRACING
    event!(target: "practical_rust_book", Level::INFO, "querying questions");
    let mut pagination = Pagination::default();

    if !params.is_empty() {
        // let pagination = extract_pagination(params)?; // BEFORE adding psql
        event!(Level::INFO, pagination = true);
        pagination = extract_pagination(params)?;
        // log::info!("{} Pagination set {:?}", id, &pagination); LOGGING
        // OLD CODE
        // info!(pagination = true); // TRACING
        // let res: Vec<Question> = store.questions.read().await.values().cloned().collect();
        // let res = &res[pagination.start..pagination.end];
        // Ok(warp::reply::json(&res))
    }

    match store
        .get_questions(pagination.limit, pagination.offset)
        .await
    {
        Ok(res) => Ok(warp::reply::json(&res)),
        Err(e) => Err(warp::reject::custom(e)),
    }
    // else {
    //     // log::info!("{} No pagination used.", id); LOGGING
    //     info!(pagination = false);
    //     let res: Vec<Question> = match store
    //         .get_questions(pagination.limit, pagination.offset)
    //         .await
    //     {
    //         Ok(res) => res,
    //         Err(e) => return Err(warp::reject::custom(Error::DatabaseQueryError)),
    //     };
    //     // .questions.read().await.values().cloned().collect();

    // }
}

pub async fn add_question(
    store: Store,
    new_question: NewQuestion,
) -> Result<impl warp::Reply, warp::Rejection> {
    match store.add_question(new_question).await {
        Ok(_) => Ok(warp::reply::with_status("Question added.", StatusCode::OK)),
        Err(e) => Err(warp::reject::custom(e)),
    }

    // store
    //     .questions
    //     .write()
    //     .await
    //     .insert(question.id.clone(), question);

    // Ok(warp::reply::with_status("Question added.", StatusCode::OK))
}

pub async fn update_question(
    id: i32,
    store: Store,
    question: Question,
) -> Result<impl warp::Reply, warp::Rejection> {
    match store.update_question(question, id).await {
        Ok(res) => Ok(warp::reply::json(&res)),
        Err(e) => Err(warp::reject::custom(e)),
    }
    // let res = match store.update_question(question, id).await {
    //     Ok(res) => res,
    //     Err(e) => return Err(warp::reject::custom(Error::DatabaseQueryError)),
    // };
    // Ok(warp::reply::json(&res))

    // match store.questions.write().await.get_mut(&QuestionId(id)) {
    //     Some(q) => *q = question,
    //     None => return Err(warp::reject::custom(Error::QuestionNotFound)),
    // }
    // Ok(warp::reply::with_status("Question udpated", StatusCode::OK))
}

pub async fn delete_question(id: i32, store: Store) -> Result<impl warp::Reply, warp::Rejection> {
    match store.delete_question(id).await {
        Ok(_) => Ok(warp::reply::with_status(
            format!("Question {} deleted", id),
            StatusCode::OK,
        )),
        Err(e) => Err(warp::reject::custom(e)),
    }
    // if let Err(e) = store.delete_question(id).await {
    //     return Err(warp::reject::custom(Error::DatabaseQueryError));
    // }
    // Ok(warp::reply::with_status(
    //     format!("Question {} deleted", id),
    //     StatusCode::OK,
    // ))

    // match store.questions.write().await.remove(&QuestionId(id)) {
    //     Some(_) => Ok(warp::reply::with_status(
    //         "Question deleted.",
    //         StatusCode::OK,
    //     )),
    //     None => Err(warp::reject::custom(Error::QuestionNotFound)),
    // }
}
