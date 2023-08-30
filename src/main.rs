#![warn(clippy::all)]
use clap::Parser;
use config::Config;
use dotenv;
use handle_errors::return_error;
use std::env;
use tracing_subscriber::fmt::format::FmtSpan;
use warp::{http::Method, Filter};

mod profanity;
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

// Q&A web service API
#[derive(Parser, Debug, Default, serde::Deserialize, PartialEq)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Which errors we want to log (info, warn or error)
    #[clap(short, long, default_value = "warn")]
    log_level: String,
    /// Which PORT the server is listening to
    #[clap(short, long, default_value = "8080")]
    port: u16,
    /// Database user
    #[clap(long, default_value = "user")]
    db_user: String,
    /// Url for postgres database
    #[clap(long, default_value = "localhost")]
    db_host: String,
    /// PORT number for database connection
    #[clap(long, default_value = "5432")]
    db_port: u16,
    /// Database naem
    #[clap(long, default_value = "rustwebdev")]
    db_name: String,
    // Web server port
    // port: u16,
}

#[tokio::main]
async fn main() -> Result<(), handle_errors::Error> {
    dotenv::dotenv().ok();

    if let Err(_) = env::var("BAD_WORDS_API_KEY") {
        panic!("BadWords API key not set.")
    }

    if let Err(_) = env::var("PASETO_KEY") {
        panic!("Paseto key not set.")
    }

    let port = std::env::var("PORT")
        .ok()
        .map(|val| val.parse::<u16>())
        .unwrap_or(Ok(8080))
        .map_err(|e| handle_errors::Error::ParseError(e))?;

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

    // ONLY IF WE USE CONFIG.toml file
    // let config = Config::builder()
    //     .add_source(config::File::with_name("setup"))
    //     .build()
    //     .unwrap();

    // let config = config.try_deserialize::<Args>().unwrap();

    // USING CLI args
    let args = Args::parse();
    // TRACING
    // let log_filter = std::env::var("RUST_LOG")
    //     .unwrap_or_else(|_| "practical_rust_book=info,warp=error".to_owned());

    let db_user = env::var("POSTGRES_USER").unwrap_or(args.db_user.to_owned());
    let db_password = env::var("POSTGRES_PASSWORD").unwrap();
    let db_host = env::var("POSTGRES_HOST").unwrap_or(args.db_host.to_owned());
    let db_port = env::var("POSTGRES_PORT").unwrap_or(args.db_port.to_string());
    let db_name = env::var("POSTGRES_DB").unwrap_or(args.db_name.to_owned());

    // after setting up setup.toml file for config variables
    // replace config with args
    let log_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| {
        format!(
            "handle_error={},rust_web_dev={},warp={}",
            args.log_level, args.log_level, args.log_level
        )
    });

    // if you need to add a username and password,
    // the connection would look like:
    // "postgres:/ /username:password@localhost:5432/rustwebdev"
    // let store = store::Store::new("postgres://localhost:5432/rustwebdev").await;

    // replace config with args
    let store = store::Store::new(&format!(
        "postgres://{}:{}@{}:{}/{}",
        db_user, db_password, db_host, db_port, db_name
    ))
    .await
    .map_err(|e| handle_errors::Error::DatabaseQueryError(e))?;

    sqlx::migrate!()
        .run(&store.clone().connection)
        .await
        .map_err(|e| handle_errors::Error::MigrationError(e))?;
    // .expect("Cannot run migration");

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
        .and(routes::authentication::auth())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::question::add_question);

    let update_question = warp::put()
        .and(warp::path("questions"))
        .and(warp::path::param::<i32>())
        .and(warp::path::end())
        .and(routes::authentication::auth())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::question::update_question);

    let delete_question = warp::delete()
        .and(warp::path("questions"))
        .and(warp::path::param::<i32>())
        .and(warp::path::end())
        .and(routes::authentication::auth())
        .and(store_filter.clone())
        .and_then(routes::question::delete_question);

    let add_answer = warp::post()
        .and(warp::path("answers"))
        .and(warp::path::end())
        .and(routes::authentication::auth())
        .and(store_filter.clone())
        .and(warp::body::form())
        .and_then(routes::answer::add_answer);

    let registration = warp::post()
        .and(warp::path("registration"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::authentication::register);

    let login = warp::post()
        .and(warp::path("login"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::authentication::login);

    let routes = get_questions
        .or(add_question)
        .or(update_question)
        .or(add_answer)
        .or(delete_question)
        .or(registration)
        .or(login)
        .with(cors)
        .with(warp::trace::request())
        .recover(return_error);

    tracing::info!("Q&A service build ID {}", env!("RUST_WEB_DEV_VERSION"));

    // warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
    // warp::serve(routes).run(([127, 0, 0, 1], config.port)).await;
    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
    Ok(())
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
