use argon2::Error as ArgonError;
use reqwest::Error as ReqwestError;
use reqwest_middleware::Error as MiddlewareReqwestError;
use warp::{
    filters::{body::BodyDeserializeError, cors::CorsForbidden},
    http::StatusCode,
    reject::Reject,
    Rejection, Reply,
};

// use sqlx::error::Error as SqlxError;
use tracing::{event, instrument, Level};

#[derive(Debug)]
pub enum Error {
    ParseError(std::num::ParseIntError),
    MissingParameters,
    WrongPassword,
    ArgonLibraryError(ArgonError),
    // QuestionNotFound,
    DatabaseQueryError(sqlx::Error),
    ReqwestAPIError(ReqwestError),
    MiddlewareReqwestAPIError(MiddlewareReqwestError),
    // ExternalAPIError(ReqwestError),
    ClientError(APILayerError),
    ServerError(APILayerError),
    CannotDecryptToken,
    Unauthorized,
}

#[derive(Debug, Clone)]
pub struct APILayerError {
    pub status: u16,
    pub message: String,
}

impl std::fmt::Display for APILayerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Status: {}, Message: {}", self.status, self.message)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &*self {
            Error::ParseError(ref err) => {
                write!(f, "Cannot parse parameter: {}", err)
            }
            Error::MissingParameters => write!(f, "Missing parameter."),
            // Error::QuestionNotFound => write!(f, "Question not found."),
            Error::WrongPassword => write!(f, "Wrong password."),
            Error::ArgonLibraryError(_) => write!(f, "Cannot verify password."),
            Error::DatabaseQueryError(_) => write!(f, "Cannot update, invalid data."),
            Error::ReqwestAPIError(err) => write!(f, "External API error: {}", err),
            Error::MiddlewareReqwestAPIError(err) => write!(f, "External API error: {}", err),
            Error::ClientError(err) => write!(f, "External client error: {}", err),
            Error::ServerError(err) => write!(f, "External server error: {}", err),
            Error::CannotDecryptToken => write!(f, "Cannot decrypt token"),
            Error::Unauthorized => write!(f, "No permission to change the underlying resource."),
        }
    }
}

impl Reject for Error {}
impl Reject for APILayerError {}

const DUPLICATE_KEY: i32 = 23505;
#[instrument]
pub async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    // println!("{:?}", r);

    if let Some(crate::Error::DatabaseQueryError(e)) = r.find() {
        event!(Level::ERROR, "Database query error");
        match e {
            sqlx::Error::Database(err) => {
                if err.code().unwrap().parse::<i32>().unwrap() == DUPLICATE_KEY {
                    Ok(warp::reply::with_status(
                        "Account already exists".to_string(),
                        StatusCode::UNPROCESSABLE_ENTITY,
                    ))
                } else {
                    Ok(warp::reply::with_status(
                        "Cannot update data".to_string(),
                        StatusCode::UNPROCESSABLE_ENTITY,
                    ))
                }
            }
            _ => Ok(warp::reply::with_status(
                "Cannot update data".to_string(),
                StatusCode::UNPROCESSABLE_ENTITY,
            )),
        }
        // Ok(warp::reply::with_status(
        //     crate::Error::DatabaseQueryError.to_string(),
        //     StatusCode::UNPROCESSABLE_ENTITY,
        // ))
    } else if let Some(crate::Error::ReqwestAPIError(e)) = r.find() {
        event!(Level::ERROR, "{}", e);
        Ok(warp::reply::with_status(
            "Internal server error ONE".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    } else if let Some(crate::Error::Unauthorized) = r.find() {
        event!(Level::ERROR, "Not matching accound id");
        Ok(warp::reply::with_status(
            "No permission to change underlying resource".to_string(),
            StatusCode::UNAUTHORIZED,
        ))
    } else if let Some(crate::Error::WrongPassword) = r.find() {
        event!(Level::ERROR, "Entered wrong password.");
        Ok(warp::reply::with_status(
            "Wrong email/password combination.".to_string(),
            StatusCode::UNAUTHORIZED,
        ))
    } else if let Some(crate::Error::MiddlewareReqwestAPIError(e)) = r.find() {
        event!(Level::ERROR, "{}", e);
        Ok(warp::reply::with_status(
            "Internal server error.".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    } else if let Some(crate::Error::ClientError(e)) = r.find() {
        event!(Level::ERROR, "{}", e);
        Ok(warp::reply::with_status(
            "Internal server error TWO".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    } else if let Some(crate::Error::ServerError(e)) = r.find() {
        event!(Level::ERROR, "{}", e);
        Ok(warp::reply::with_status(
            "Internal server error THREE".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    } else if let Some(error) = r.find::<CorsForbidden>() {
        event!(Level::ERROR, "CORS forbidden error: {}", error);
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::FORBIDDEN,
        ))
    } else if let Some(error) = r.find::<BodyDeserializeError>() {
        event!(Level::ERROR, "Cannot deserizalize request body: {}", error);
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(error) = r.find::<Error>() {
        event!(Level::ERROR, "{}", error);
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else {
        event!(Level::WARN, "Requested route was not found");
        Ok(warp::reply::with_status(
            "Route not found".to_string(),
            StatusCode::NOT_FOUND,
        ))
    }
    // CODE BEFORE IMPLEMENTING ERROR & tracing, instead of using sqlx Error
    // if let Some(error) = r.find::<Error>() {
    //     Ok(warp::reply::with_status(
    //         error.to_string(),
    //         StatusCode::RANGE_NOT_SATISFIABLE,
    //     ))
    // } else if let Some(error) = r.find::<CorsForbidden>() {
    //     Ok(warp::reply::with_status(
    //         // "No valid ID presented",
    //         // StatusCode::UNPROCESSABLE_ENTITY,
    //         error.to_string(),
    //         StatusCode::FORBIDDEN,
    //     ))
    // } else if let Some(error) = r.find::<BodyDeserializeError>() {
    //     Ok(warp::reply::with_status(
    //         error.to_string(),
    //         StatusCode::UNPROCESSABLE_ENTITY,
    //     ))
    // } else {
    //     Ok(warp::reply::with_status(
    //         "Route not found".to_string(),
    //         StatusCode::NOT_FOUND,
    //     ))
    // }
}
