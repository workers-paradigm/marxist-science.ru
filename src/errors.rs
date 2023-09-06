use rocket::tokio::io;
use rocket::{
    http::Status,
    response::{self, Responder},
    serde::json::serde_json,
    Request,
};
use rocket_db_pools::deadpool_postgres::tokio_postgres;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Errors {
    // errors all hapen only here
    #[error("{0}")]
    Db(#[from] tokio_postgres::Error),
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("{0}")]
    Hex(#[from] hex::FromHexError),
    #[error("Not Found")]
    NotFound,
    #[error("Serialization failure: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("{0}")]
    Validation(String),
    #[error("Unauthorized")]
    Authorization,
    #[error("{0}")]
    Custom(String, Status),
}

impl<'r> Responder<'r, 'static> for Errors {
    fn respond_to(self, _request: &'r Request<'_>) -> response::Result<'static> {
        let status = match &self {
            Self::Db(_) | Self::Io(_) | Self::Serialization(_) => Status::InternalServerError,
            Self::Hex(_) => Status::BadRequest,
            Self::NotFound => Status::NotFound,
            Self::Validation(_) => Status::UnprocessableEntity,
            Self::Authorization => Status::Unauthorized,
            Self::Custom(_, status) => *status,
        };

        // When I'll have anything to respond to the user, I will.
        // Response::build()
        //     .header(ContentType::Plain)
        //     .status(status)
        //     .sized_body(message.len(), Cursor::new(message))
        //     .ok()

        warn_!("{self}");
        Err(status)
    }
}
