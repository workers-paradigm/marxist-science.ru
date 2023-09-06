use crate::{
    dashboard,
    db::{Connection, Postgres},
    errors::Errors,
};
use rocket::form::Form;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::outcome::try_outcome;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::response::Redirect;
use rocket_dyn_templates::{context, Template};

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordVerifier},
    Argon2,
};
use rand::RngCore;

pub struct AdminMaybe(bool);
pub struct AdminOnly;

async fn session_exists(db: Connection<Postgres>, session: &str) -> Result<bool, Errors> {
    db.query_one(
        "SELECT EXISTS (SELECT 1 FROM sessions WHERE id = $1::bytea AND expires_at > NOW())",
        &[&hex::decode(session)?],
    )
    .await?
    .try_get::<usize, bool>(0)
    .map_err(|e| e.into())
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminMaybe {
    type Error = Errors;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Errors> {
        let session = req.cookies().get_private("session");
        if let None = session {
            return Outcome::Success(Self(false));
        }
        let session = session.unwrap();
        match req.guard::<Connection<Postgres>>().await {
            Outcome::Success(db) => match session_exists(db, session.value()).await {
                Ok(boolean) => Outcome::Success(Self(boolean)),
                Err(error) => Outcome::Failure((Status::InternalServerError, error)),
            },
            Outcome::Failure(_) | Outcome::Forward(_) => Outcome::Failure((
                Status::InternalServerError,
                Errors::Custom(
                    "Failed to connect to database!".to_owned(),
                    Status::InternalServerError,
                ),
            )),
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminOnly {
    type Error = Errors;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Errors> {
        if try_outcome!(req.guard::<AdminMaybe>().await).0 {
            Outcome::Success(Self)
        } else {
            Outcome::Failure((Status::Unauthorized, Errors::Authorization))
        }
    }
}

#[rocket::get("/login")]
pub async fn login(admin: AdminMaybe) -> Result<Template, Redirect> {
    if admin.0 {
        Err(Redirect::to("/dashboard"))
    } else {
        Ok(Template::render("login", context! {}))
    }
}

#[derive(FromForm)]
pub struct Login {
    username: String,
    password: String,
}

#[rocket::post("/login", data = "<login_data>")]
pub async fn do_log_in(
    jar: &CookieJar<'_>,
    login_data: Form<Login>,
    admin: AdminMaybe,
    db: Connection<Postgres>,
) -> Result<Redirect, Errors> {
    if admin.0 {
        return Ok(Redirect::to(uri![dashboard::index]));
    }

    let (id, hash): (i32, String) = db
        .query_opt(
            "SELECT id, password_hash FROM users WHERE username = $1",
            &[&login_data.username],
        )
        .await?
        .ok_or(Errors::Authorization)
        .map(|row| {
            Ok::<_, Errors>((
                row.try_get::<usize, i32>(0)?,
                row.try_get::<usize, String>(1)?,
            ))
        })??;

    if Argon2::default()
        .verify_password(
            login_data.password.as_bytes(),
            &PasswordHash::new(hash.as_str()).unwrap(),
        )
        .is_ok()
    {
        let mut session = [0u8; 16];
        OsRng.fill_bytes(&mut session);
        jar.add_private(Cookie::new("session", hex::encode(session)));
        db.execute(
            "INSERT INTO sessions (user_id, id) VALUES ($1, $2)",
            &[&id, &&session[..]],
        )
        .await?;
        Ok(Redirect::to(uri![dashboard::index]))
    } else {
        Err(Errors::Authorization)
    }
}
