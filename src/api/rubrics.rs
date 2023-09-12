use crate::api::{
    article,
    upload::{path_to, upload_file},
};
use crate::auth::AdminOnly;
use crate::db::{Connection, Postgres};
use crate::errors::Errors;
use crate::models::{
    article::ArticleForm,
    file::HashedFileBuf,
    rubrics::{self, Rubric},
};
use rocket::form::Form;
use rocket::response::Redirect;
use rocket_dyn_templates::{context, Template};

#[rocket::post("/rubrics/create")]
pub async fn create(_admin: AdminOnly, db: Connection<Postgres>) -> Result<Redirect, Errors> {
    let title = "NEED RUBRIC TITLE".to_owned();
    let id = db
        .query_one(
            "INSERT INTO rubrics (title) VALUES ($1) RETURNING id",
            &[&title],
        )
        .await?
        .try_get::<usize, i32>(0)?;
    Ok(Redirect::to(uri![get(id)]))
}

#[rocket::put("/rubrics/update", data = "<rubric>")]
pub async fn update(
    _admin: AdminOnly,
    db: Connection<Postgres>,
    rubric: Form<Rubric>,
) -> Result<Redirect, Errors> {
    db.execute(
        "UPDATE rubrics SET title = $2 WHERE id = $1",
        &[&rubric.id, &rubric.title],
    )
    .await?;
    Ok(Redirect::to(uri![get(rubric.id)]))
}

#[rocket::get("/rubrics/get?<id>")]
pub async fn get(db: Connection<Postgres>, id: i32) -> Result<Template, Errors> {
    let rubric = db
        .query_opt("SELECT id, title, cover FROM rubrics WHERE id = $1", &[&id])
        .await?
        .map_or(Err(Errors::NotFound), |row| {
            Ok(Rubric {
                id: row.try_get(0)?,
                title: row.try_get(1)?,
                cover: row.try_get(2)?,
            })
        })?;
    Ok(Template::render("htmx/rubrics/rubric", context! { rubric }))
}

#[rocket::delete("/rubrics/delete", data = "<rubric>")]
pub async fn delete(
    _admin: AdminOnly,
    db: Connection<Postgres>,
    rubric: Form<Rubric>,
) -> Result<(), Errors> {
    db.execute("DELETE FROM rubrics WHERE id = $1", &[&rubric.id])
        .await?;
    Ok(())
}

#[rocket::get("/rubrics/<id>")]
pub async fn index(db: Connection<Postgres>, id: i32) -> Result<Template, Errors> {
    let title = db
        .query_opt("SELECT title FROM rubrics WHERE id = $1", &[&id])
        .await?
        .map_or(Err(Errors::NotFound), |row| {
            row.try_get::<usize, String>(0).map_err(|e| e.into())
        })?;
    db.query("SELECT a.id, a.title, a.author, a.cover FROM rubrics as r JOIN articles_rubrics as ar ON ar.rubric = r.id JOIN articles as a ON ar.article = a.id AND a.published WHERE r.id = $1", &[&id])
        .await?
        .iter()
        .map(|row| Ok(ArticleForm {
            id: row.try_get(0)?,
            title: row.try_get(1)?,
            author: row.try_get(2)?,
            cover: row.try_get(3)?,
            rubrics: vec![],
            published: true,
        }))
        .collect::<Result<Vec<_>, _>>()
        .map(|vec| Template::render("rubrics/index", context! { title, entries: vec }))
}

#[rocket::get("/rubrics/list-for-picker?<article_id>")]
pub async fn list_for_picker(
    db: Connection<Postgres>,
    article_id: i32,
) -> Result<Template, Errors> {
    rubrics::get_all(&db).await.map(|rubrics| {
        Template::render(
            "htmx/rubrics/list-for-picker",
            context! { rubrics, id: article_id },
        )
    })
}

#[rocket::put("/rubrics/attach-rubric?<rubric_id>&<article_id>")]
pub async fn attach_rubric(
    _admin: AdminOnly,
    db: Connection<Postgres>,
    rubric_id: i32,
    article_id: i32,
) -> Result<Redirect, Errors> {
    db.execute(
        "INSERT INTO articles_rubrics (article, rubric) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        &[&article_id, &rubric_id],
    )
    .await?;
    Ok(Redirect::to(uri![article::rubrics_of_article(article_id)]))
}

#[rocket::delete("/rubrics/detach-rubric?<rubric_id>&<article_id>")]
pub async fn detach_rubric(
    _admin: AdminOnly,
    db: Connection<Postgres>,
    rubric_id: i32,
    article_id: i32,
) -> Result<Redirect, Errors> {
    db.execute(
        "DELETE FROM articles_rubrics WHERE article = $1 AND rubric = $2",
        &[&article_id, &rubric_id],
    )
    .await?;
    Ok(Redirect::to(uri![article::rubrics_of_article(article_id)]))
}

#[derive(FromForm)]
pub struct FileForm {
    file: HashedFileBuf,
}

#[rocket::put("/rubrics/attach-image?<rubric>", data = "<image>")]
pub async fn attach_image(
    _admin: AdminOnly,
    mut db: Connection<Postgres>,
    rubric: i32,
    mut image: Form<FileForm>,
) -> Result<Redirect, Errors> {
    let trans = db.transaction().await?;
    upload_file(&trans, &mut image.file).await?;
    let location = path_to(
        &hex::encode(image.file.hash),
        &image.file.content_type.extension_err()?,
    );
    trans
        .execute(
            "UPDATE rubrics SET cover = $1 WHERE id = $2",
            &[&location.as_path().to_str(), &rubric],
        )
        .await?;
    trans.commit().await?;
    Ok(Redirect::to(uri![get(rubric)]))
}
