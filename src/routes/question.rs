#![warn(clippy::all)]

use std::alloc::handle_alloc_error;
use std::collections::HashMap;

use tracing::{event, info, instrument, Level};
use warp::http::StatusCode;

use serde::{Deserialize, Serialize};

use crate::store::Store;
use crate::types::pagination::{self, extract_pagination, Pagination};
use crate::types::question::{self, NewQuestion, Question, QuestionId};
// use handle_errors::Error;
use crate::profanity::check_profanity;
use crate::types::account::Session;

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
    session: Session,
    store: Store,
    new_question: NewQuestion,
) -> Result<impl warp::Reply, warp::Rejection> {
    let account_id = session.account_id;

    let title = match check_profanity(new_question.title).await {
        Ok(res) => {
            println!("{:?}", res);
            res
        }
        Err(e) => return Err(warp::reject::custom(e)),
    };

    let content = match check_profanity(new_question.content).await {
        Ok(res) => {
            println!("{:?}", res);
            res
        }
        Err(e) => return Err(warp::reject::custom(e)),
    };

    // OLD CODE BEFORE PROFANITY.RS FILE
    // let client = reqwest::Client::new();
    // let res = client
    //     .post("https:/ /api.apilayer.com/bad_words?censor_character=*")
    //     .header("apikey", "JvpKmotUBxfl5n8OxmTOwfrmZrtAXKNI")
    //     .body(new_question.content)
    //     .send()
    //     .await
    //     .map_err(|e| handle_errors::Error::ExternalAPIError(e))?;
    // // .text()
    // // .await
    // // .map_err(|e| handle_errors::Error::ExternalAPIError(e))?;
    // if !res.status().is_success() {
    //     if res.status().is_client_error() {
    //         let err = transform_error(res).await;
    //         return Err(handle_errors::Error::ClientError(err).into());
    //     } else {
    //         let err = transform_error(res).await;
    //         return Err(handle_errors::Error::ServerError(err).into());
    //     }
    // }

    // BEFORE ADDING TYPES FOR API RESPONSES
    // match res.error_for_status() {
    //     Ok(res) => {
    //         let res = res
    //             .text()
    //             .await
    //             .map_err(|e| handle_errors::Error::ExternalAPIError(e))?;
    //         println!("{}", res);

    //         match store.add_question(new_question).await {
    //             Ok(_) => Ok(warp::reply::with_status("Question added.", StatusCode::OK)),
    //             Err(e) => Err(warp::reject::custom(e)),
    //         }
    //     }
    //     Err(err) => Err(warp::reject::custom(
    //         handle_errors::Error::ExternalAPIError(err),
    //     )),
    // }

    // let res = res
    //     .json::<BadWordsResponse>()
    //     .await
    //     .map_err(|e| handle_errors::Error::ExternalAPIError(e))?;
    // let content = res.censored_content;

    let question = NewQuestion {
        title,
        content,
        tags: new_question.tags,
    };
    match store.add_question(question, account_id).await {
        Ok(question) => Ok(warp::reply::json(&question)),
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
    session: Session,
    store: Store,
    question: Question,
) -> Result<impl warp::Reply, warp::Rejection> {
    let account_id = session.account_id;
    if store.is_question_owner(id, &account_id).await? {
        let title = check_profanity(question.title);
        // let title = match check_profanity(question.title).await {
        //     Ok(res) => res,
        //     Err(e) => return Err(warp::reject::custom(e)),
        // };

        let content = check_profanity(question.content);
        // let content = match check_profanity(question.content).await {
        //     Ok(res) => res,
        //     Err(e) => return Err(warp::reject::custom(e)),
        // };

        let (title, content) = tokio::join!(title, content);
        if title.is_ok() && content.is_ok() {
            // if title.is_err() {
            //     return Err(warp::reject::custom(title.unwrap_err()));
            // }

            // if content.is_err() {
            //     return Err(warp::reject::custom(content.unwrap_err()));
            // }

            let question = Question {
                id: question.id,
                title: title.unwrap(),
                content: content.unwrap(),
                tags: question.tags,
            };

            match store.update_question(question, id, account_id).await {
                Ok(res) => Ok(warp::reply::json(&res)),
                Err(e) => Err(warp::reject::custom(e)),
            }
        } else {
            Err(warp::reject::custom(
                title.expect_err("Expected API call to have failed here."),
            ))
        }
    } else {
        Err(warp::reject::custom(handle_errors::Error::Unauthorized))
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

pub async fn delete_question(
    id: i32,
    session: Session,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let account_id = session.account_id;

    if store.is_question_owner(id, &account_id).await? {
        match store.delete_question(id, &account_id).await {
            Ok(_) => Ok(warp::reply::with_status(
                format!("Question {} deleted", id),
                StatusCode::OK,
            )),
            Err(e) => Err(warp::reject::custom(e)),
        }
    } else {
        Err(warp::reject::custom(handle_errors::Error::Unauthorized))
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
