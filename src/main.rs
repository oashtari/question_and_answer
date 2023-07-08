use std::io::{Error, ErrorKind};
use std::str::FromStr;

use serde::Serialize;
use std::collections::HashMap;
use warp::cors::CorsForbidden;
use warp::{http::Method, http::StatusCode, reject::Reject, Filter, Rejection, Reply};
#[derive(Debug, Serialize)]
struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}
#[derive(Debug, Serialize)]
struct QuestionId(String);

impl Question {
    fn new(id: QuestionId, title: String, content: String, tags: Option<Vec<String>>) -> Self {
        Question {
            id,
            title,
            content,
            tags,
        }
    }
}

impl FromStr for QuestionId {
    type Err = std::io::Error;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        match id.is_empty() {
            false => Ok(QuestionId(id.to_string())),
            true => Err(Error::new(ErrorKind::InvalidInput, "No id provided")),
        }
    }
}

#[derive(Debug)]
struct InvalidId;
impl Reject for InvalidId {}

async fn get_questions() -> Result<impl warp::Reply, warp::Rejection> {
    let question = Question::new(
        QuestionId::from_str("1").expect("No id provided"),
        "First Question".to_string(),
        "Content of question".to_string(),
        Some(vec!["FAQ".to_string()]),
    );

    match question.id.0.parse::<i32>() {
        Err(_) => Err(warp::reject::custom(InvalidId)),
        Ok(_) => Ok(warp::reply::json(&question)),
    }
}

async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    // println!("{:?}", r);
    if let Some(InvalidId) = r.find() {
        Ok(warp::reply::with_status(
            "No valid ID presented",
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else {
        Ok(warp::reply::with_status(
            "Route not found",
            StatusCode::NOT_FOUND,
        ))
    }
}

#[tokio::main]
async fn main()
// -> Result<(), Box<dyn std::error::Error>>
{
    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(&[Method::PUT, Method::DELETE, Method::POST, Method::GET]);

    let get_items = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and_then(get_questions)
        .recover(return_error);

    let routes = get_items.with(cors);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;

    // Ok(())
}

// let question = Question::new(
//     QuestionId::from_str("1").expect("No id provided"),
//     "First Question".to_string(),
//     "Content of question".to_string(),
//     Some(vec!["faq".to_string()]),
// );

// let question2 = Question::new(
//     QuestionId::from_str("2").expect("No id provided"),
//     "Second Question".to_string(),
//     "Content of next question".to_string(),
//     Some(vec!["FAQ".to_string()]),
// );
// println!("{:?}", question);

// let resp = reqwest::get("https:/ /httpbin.org/ip")
//     .await?
//     .json::<HashMap<String, String>>()
//     .await?;
// println!("{:#?}", resp);

// let hello = warp::path("hello")
//     .and(warp::path::param())
//     .map(|name: String| format!("Hello, {}!", name));

// warp::serve(hello).run(([127, 0, 0, 1], 1337)).await;

// #[tokio::main]
// async fn main() {
//     // create a path Filter
//     let hello = warp::path("hello").map(|| format!("Hello, World!"));

//     // start the server and pass the route filter to it
//     warp::serve(hello).run(([127, 0, 0, 1], 3030)).await;
// }
