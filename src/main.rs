#[macro_use]
extern crate rocket;

use rocket::fs::{relative, FileServer};
use rocket::Config;
use rocket_dyn_templates::{context, Template};

mod api;
mod auth;
mod errors;
mod models;

use api::{archive, article, dashboard, rubrics, upload};
use db::{Connection, Database, Postgres};
use errors::Errors;

#[rocket::get("/")]
pub async fn index(db: Connection<Postgres>) -> Result<Template, Errors> {
    let rubrics = models::rubrics::get_populated(&db, Some(6)).await?;
    let articles = models::article::get_published(&db, Some(9)).await?;
    Ok(Template::render("index", context! { rubrics, articles }))
}

#[rocket::launch]
fn rocket() -> _ {
    dotenvy::from_filename(relative![".env"]).unwrap();

    let figment = Config::figment()
        .merge((
            "secret_key",
            dotenvy::var("ROCKET_COOKIE_KEY")
                .expect("ROCKET_COOKIE_KEY must be set in the .env file"),
        ))
        .merge((
            "databases.postgres.url",
            dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set in the .env file"),
        ))
        .merge(("template_dir", relative!["templates"]));

    rocket::custom(figment)
        .mount(
            "/",
            routes![
                index,
                article::index,
                article::edit,
                article::save,
                article::contents,
                article::create,
                article::view,
                article::save_info,
                article::delete,
                article::rubrics_of_article,
                archive::index,
                archive::attach_files,
                archive::detach_files,
                archive::update_entry_info,
                archive::create_entry,
                rubrics::create,
                rubrics::get,
                rubrics::update,
                rubrics::delete,
                rubrics::index,
                rubrics::list_for_picker,
                rubrics::attach_rubric,
                rubrics::detach_rubric,
                rubrics::attach_image,
                upload::upload_one,
                upload::files,
                upload::get,
                upload::delete,
                dashboard::index,
                dashboard::articles,
                dashboard::archive,
                dashboard::materials,
                dashboard::delete_entry,
                dashboard::rubrics,
                auth::login,
                auth::do_log_in,
            ],
        )
        .mount("/static", FileServer::from(relative!("static")).rank(11))
        .mount("/static/js", FileServer::from(relative!("js/dist")))
        .attach(Postgres::init())
        .attach(Template::fairing())
}

mod db {
    use rocket_db_pools::deadpool_postgres::Pool;

    pub use rocket_db_pools::{
        deadpool_postgres::tokio_postgres::{self, Statement, Transaction},
        Connection, Database,
    };

    #[derive(Database)]
    #[database("postgres")]
    pub struct Postgres(Pool);
}

// can use FileServer for static files

// The rocket cycle:
// 1. Match handlers (static > dynamic)
// 2. Run it ig
// 3. If Responder (handler return trait) returns Err, dispatch to Catcher
// 4. ???
// 5. Profit
