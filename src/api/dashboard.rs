use crate::{
    api::archive::fetch_all_entries,
    auth::AdminOnly,
    context,
    db::{Connection, Postgres},
    errors::Errors,
    models::{article, rubrics as rubrics_mod, FileRecord},
    Template,
};

#[rocket::get("/dashboard/rubrics")]
pub async fn rubrics(_admin: AdminOnly, db: Connection<Postgres>) -> Result<Template, Errors> {
    rubrics_mod::get_all(&db)
        .await
        .map(|rubrics| Template::render("dashboard/rubrics", context! { rubrics }))
}

#[rocket::get("/dashboard")]
pub async fn index(_admin: AdminOnly, db: Connection<Postgres>) -> Result<Template, Errors> {
    db.query("SELECT id, title, ext FROM files", &[])
        .await?
        .iter()
        .map(|row| {
            Ok(FileRecord::from(
                row.try_get(0)?,
                row.try_get(1)?,
                row.try_get(2)?,
            ))
        })
        .collect::<Result<Vec<FileRecord>, _>>()
        .map(|files| Template::render("dashboard/index", context! { files }))
}

#[rocket::get("/dashboard/archive")]
pub async fn archive(_admin: AdminOnly, db: Connection<Postgres>) -> Result<Template, Errors> {
    fetch_all_entries(&db)
        .await
        .map(|entries| Template::render("dashboard/archive", context! { entries }))
}

#[rocket::get("/dashboard/articles")]
pub async fn articles(_admin: AdminOnly, db: Connection<Postgres>) -> Result<Template, Errors> {
    article::get_all(&db)
        .await
        .map(|entries| Template::render("dashboard/articles", context! { entries }))
}

#[rocket::get("/dashboard/materials")]
pub async fn materials(db: Connection<Postgres>) -> Result<Template, Errors> {
    db.query("SELECT id, title, ext FROM files", &[])
        .await?
        .iter()
        .map(|row| {
            Ok(FileRecord::from(
                row.try_get(0)?,
                row.try_get(1)?,
                row.try_get(2)?,
            ))
        })
        .collect::<Result<Vec<FileRecord>, Errors>>()
        .map(|records| Template::render("dashboard/materials", context! { records }))
        .map_err(|e| e.into())
}

#[rocket::delete("/archive/delete-entry?<id>")]
pub async fn delete_entry(
    _admin: AdminOnly,
    db: Connection<Postgres>,
    id: i32,
) -> Result<(), Errors> {
    db.execute("DELETE FROM archive_entries WHERE id = $1", &[&id])
        .await?;
    Ok(())
}
