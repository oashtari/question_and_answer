use futures_util::future::FutureExt;
use question_and_answer::{config, handle_errors, oneshot, setup_store};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, Write};
use std::process::Command;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct User {
    email: String,
    password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Question {
    title: String,
    content: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct QuestionAnswer {
    id: i32,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

#[tokio::main]
async fn main() -> Result<(), handle_errors::Error> {
    dotenv::dotenv().ok();
    let config = config::Config::new().expect("Config can't be set.");

    let s = Command::new("sqlx")
        .arg("database")
        .arg("drop")
        .arg("--database-url")
        .arg(format!(
            "postgres://{}:{}/{}",
            config.db_host, config.db_port, config.db_name
        ))
        .arg("-y")
        .output()
        .expect("sqlx command failed to start");

    io::stdout().write_all(&s.stderr).unwrap();

    let s = Command::new("sqlx")
        .arg("database")
        .arg("create")
        .arg("--database-url")
        .arg(format!(
            "postgres://{}:{}/{}",
            config.db_host, config.db_port, config.db_name
        ))
        .output()
        .expect("sqlx command failed to start");

    io::stdout().write_all(&s.stderr).unwrap();

    let store = setup_store(&config).await?;

    let handler = oneshot(store).await;

    let u = User {
        email: "test@email.com".to_string(),
        password: "password".to_string(),
    };

    register_new_user(&u).await;

    print!("Running register_new_user...");

    let result = std::panic::AssertUnwindSafe(register_new_user(&u))
        .catch_unwind()
        .await;

    match result {
        Ok(_) => println!("CHECK MARK"),
        Err(_) => {
            let _ = handler.sender.send(1);
            std::process::exit(1);
        }
    }

    print!("Running login...");
    match std::panic::AssertUnwindSafe(login(u)).catch_unwind().await {
        Ok(t) => {
            token = t;
            println!("✓");
        }
        Err(_) => {
            let _ = handler.sender.send(1);
            std::process::exit(1);
        }
    }

    print!("Running post_question...");
    match std::panic::AssertUnwindSafe(post_question(token))
        .catch_unwind()
        .await
    {
        Ok(_) => println!("✓"),
        Err(_) => {
            let _ = handler.sender.send(1);
            std::process::exit(1);
        }
    }

    let _ = handler.sender.send(1);

    Ok(())
}

async fn register_new_user(user: &User) {
    let client = reqwest::Client::new();
    let res = client
        .post("http://localhost:3030/registration")
        .json(&user)
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();

    assert_eq!(res, "Account added".to_string());
}

async fn login(user: User) -> Token {
    let client = reqwest::Client::new();
    let res = client
        .post("http://localhost:3030/login")
        .json(&user)
        .send()
        .await
        .unwrap();
}

async fn post_question(token: Token) {
    let q = Question {
        title: "First Question".to_string(),
        content: "How can I test?".to_string(),
    };

    let client = reqwest::Client::new();
    let res = client
        .post("http://localhost:3030/questions")
        .header("Authorization", token.0)
        .json(&q)
        .send()
        .await
        .unwrap()
        .json::<QuestionAnswer>()
        .await
        .unwrap();

    assert_eq!(res.id, 1);
    assert_eq!(res.title, q.title);
}
