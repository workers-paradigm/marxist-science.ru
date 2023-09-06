#[macro_use]
extern crate rocket;

use rocket::fs::{relative, FileServer};
use rocket::Config;
use rocket_dyn_templates::{context, Template};

mod api;
mod auth;
mod errors;
mod models;

use api::{archive, article, dashboard, upload};
use db::{Database, Postgres};

#[rocket::get("/")]
fn index() -> Template {
    Template::render("index", context! {})
}

#[rocket::launch]
fn rocket() -> _ {
    dotenvy::dotenv().unwrap();

    let figment = Config::figment().merge((
        "secret_key",
        dotenvy::var("ROCKET_COOKIE_KEY").expect("ROCKET_COOKIE_KEY must be set."),
    ));

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
                archive::index,
                archive::attach_files,
                archive::detach_files,
                archive::update_entry_info,
                archive::create_entry,
                upload::upload_one,
                upload::files,
                upload::get,
                upload::delete,
                dashboard::index,
                dashboard::articles,
                dashboard::archive,
                dashboard::materials,
                dashboard::delete_entry,
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
