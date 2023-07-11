use std::collections::HashMap;
use std::hash::Hash;
// use std::io::{Error, ErrorKind};
// use std::str::FromStr;

use serde::{Deserialize, Serialize};
use warp::cors::CorsForbidden;
use warp::{http::Method, http::StatusCode, reject::Reject, Filter, Rejection, Reply};

#[derive(Clone)]
struct Store {
    questions: HashMap<QuestionId, Question>,
}

impl Store {
    fn new() -> Self {
        Store {
            questions: HashMap::new(),
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
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}
#[derive(Debug, Serialize, Eq, PartialEq, Hash, Clone, Deserialize)]
struct QuestionId(String);

#[derive(Debug)]
enum Error {
    ParseError(std::num::ParseIntError),
    MissingParameters,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::ParseError(ref err) => {
                write!(f, "Cannot parse parameter: {}", err)
            }
            Error::MissingParameters => write!(f, "Missing parameter."),
        }
    }
}

impl Reject for Error {}

#[derive(Debug)]
struct Pagination {
    start: usize,
    end: usize,
}
// impl Question {
//     fn new(id: QuestionId, title: String, content: String, tags: Option<Vec<String>>) -> Self {
//         Question {
//             id,
//             title,
//             content,
//             tags,
//         }
//     }
// }

// impl FromStr for QuestionId {
//     type Err = std::io::Error;

//     fn from_str(id: &str) -> Result<Self, Self::Err> {
//         match id.is_empty() {
//             false => Ok(QuestionId(id.to_string())),
//             true => Err(Error::new(ErrorKind::InvalidInput, "No id provided")),
//         }
//     }
// }

// #[derive(Debug)]
// struct InvalidId;
// impl Reject for InvalidId {}

async fn get_questions(
    params: HashMap<String, String>,
    store: Store,
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

    if !params.is_empty() {
        let pagination = extract_pagination(params)?;
        let res: Vec<Question> = store.questions.values().cloned().collect();
        let res = &res[pagination.start..pagination.end];
        Ok(warp::reply::json(&res))
    } else {
        let res: Vec<Question> = store.questions.values().cloned().collect();
        Ok(warp::reply::json(&res))
    }
}

async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    // println!("{:?}", r);
    if let Some(error) = r.find::<Error>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::RANGE_NOT_SATISFIABLE,
        ))
    } else if let Some(error) = r.find::<CorsForbidden>() {
        Ok(warp::reply::with_status(
            // "No valid ID presented",
            // StatusCode::UNPROCESSABLE_ENTITY,
            error.to_string(),
            StatusCode::FORBIDDEN,
        ))
    } else {
        Ok(warp::reply::with_status(
            "Route not found".to_string(),
            StatusCode::NOT_FOUND,
        ))
    }
}

fn extract_pagination(params: HashMap<String, String>) -> Result<Pagination, Error> {
    if params.contains_key("start") && params.contains_key("end") {
        return Ok(Pagination {
            start: params
                .get("start")
                .unwrap()
                .parse::<usize>()
                .map_err(Error::ParseError)?,
            end: params
                .get("end")
                .unwrap()
                .parse::<usize>()
                .map_err(Error::ParseError)?,
        });
    }
    Err(Error::MissingParameters)
}
#[tokio::main]
async fn main()
// -> Result<(), Box<dyn std::error::Error>>
{
    let store = Store::new();
    let store_filter = warp::any().map(move || store.clone());

    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(&[Method::PUT, Method::DELETE, Method::POST, Method::GET]);

    let get_questions = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(warp::query())
        .and(store_filter)
        .and_then(get_questions)
        .recover(return_error);

    let routes = get_questions.with(cors);

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
