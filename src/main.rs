#![warn(clippy::all)]
use handle_errors::return_error;
use tracing_subscriber::fmt::format::FmtSpan;
use warp::{http::Method, Filter};

mod routes;
mod store;
mod types;

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

#[tokio::main]
async fn main()
// -> Result<(), Box<dyn std::error::Error>>
{
    // LOGGING
    // log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    // log::error!("This is an error");
    // log::info!("This is info!");
    // log::warn!("This is a warning.");

    // let log = warp::log::custom(|info| {
    //     log::info!(
    //         "{} {} {} {:?} from {} with {:?}",
    //         info.method(),
    //         info.path(),
    //         info.status(),
    //         info.elapsed(),
    //         info.remote_addr().unwrap(),
    //         info.request_headers()
    //     );
    // });

    // TRACING
    let log_filter = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "practical_rust_book=info,warp=error".to_owned());

    let store = store::Store::new();
    let store_filter = warp::any().map(move || store.clone());

    // LOGGING
    // let id_filter = warp::any().map(|| uuid::Uuid::new_v4().to_string());

    tracing_subscriber::fmt()
        // Use the filter we built above to determine which traces we record.
        .with_env_filter(log_filter)
        // Record an even when each span closes.
        // This can be used to time our routes' destinations!
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(&[Method::PUT, Method::DELETE, Method::POST, Method::GET]);

    let get_questions = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(warp::query())
        .and(store_filter.clone())
        // .and(id_filter)  LOGGING
        .and_then(routes::question::get_questions)
        .with(warp::trace(|info| {
            tracing::info_span!(
                "get_questions request",
                method = %info.method(),
                path = %info.path(),
                id = %uuid::Uuid::new_v4()
            )
        }));
    // .recover(return_error); NOW in the routes instead of the function

    let add_question = warp::post()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::question::add_question);

    let update_question = warp::put()
        .and(warp::path("questions"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::question::update_question);

    let delete_question = warp::delete()
        .and(warp::path("questions"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::question::delete_question);

    let add_answer = warp::post()
        .and(warp::path("answers"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::form())
        .and_then(routes::answer::add_answer);

    let routes = get_questions
        .or(add_question)
        .or(update_question)
        .or(add_answer)
        .or(delete_question)
        .with(cors)
        .with(warp::trace::request())
        .recover(return_error);

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
