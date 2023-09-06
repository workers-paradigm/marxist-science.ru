use crate::{
    auth::AdminOnly,
    db::{Connection, Postgres},
    errors::Errors,
    models::FileRecord,
};
use rocket::{
    form::Form,
    serde::{Deserialize, Serialize},
};
use rocket_dyn_templates::{context, Template};

#[derive(Debug, Serialize, Deserialize, FromForm)]
#[serde(crate = "rocket::serde")]
pub struct Entry {
    id: i32,
    title: String,
    author: String,
    cover: Option<String>,
    description: String,
    files: Vec<FileRecord>,
}

#[rocket::get("/archive")]
pub async fn index(db: Connection<Postgres>) -> Result<Template, Errors> {
    Ok(Template::render(
        "archive/index",
        context! { entries: fetch_all_entries(&db).await?},
    ))
}

pub async fn fetch_entry_files(
    id: i32,
    db: &Connection<Postgres>,
) -> Result<Vec<FileRecord>, Errors> {
    db.query(
        "SELECT files.id, files.title, files.ext FROM files
         JOIN archive_entries_files AS ae ON ae.file = files.id
         WHERE ae.entry = $1",
        &[&id],
    )
    .await?
    .iter()
    .map(|row| {
        Ok(FileRecord {
            hash: hex::encode(&row.try_get::<usize, Vec<u8>>(0)?),
            name: row.try_get(1)?,
            ext: row.try_get(2)?,
        })
    })
    .collect()
}

pub async fn fetch_all_entries(db: &Connection<Postgres>) -> Result<Vec<Entry>, Errors> {
    let entry_files_query = db
        .prepare(
            "SELECT files.id, files.title, files.ext FROM files
             JOIN archive_entries_files AS ae ON ae.file = files.id
             WHERE ae.entry = $1",
        )
        .await?;

    let mut entries = db
        .query(
            "SELECT id, title, author, cover, description FROM archive_entries
             ORDER BY created_at DESC, title",
            &[],
        )
        .await?
        .iter()
        .map(|row| {
            Ok(Entry {
                id: row.try_get(0)?,
                title: row.try_get(1)?,
                author: row.try_get(2)?,
                cover: row.try_get(3)?,
                description: row.try_get(4)?,
                files: vec![],
            })
        })
        .collect::<Result<Vec<Entry>, Errors>>()?;

    for entry in &mut entries {
        entry.files = db
            .query(&entry_files_query, &[&entry.id])
            .await?
            .iter()
            .map(|row| {
                Ok(FileRecord {
                    hash: hex::encode(&row.try_get::<usize, Vec<u8>>(0)?), // ef.file
                    name: row.try_get(1)?,                                 // e.title
                    ext: row.try_get(2)?,                                  // e.ext
                })
            })
            .collect::<Result<Vec<FileRecord>, Errors>>()?;
    }
    Ok(entries)
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
#[derive(FromForm)]
pub struct EntryHashes {
    entry: i32,
    hashes: Vec<String>,
}

#[rocket::post("/archive/attach-files", data = "<pair>")]
pub async fn attach_files(
    _admin: AdminOnly,
    db: Connection<Postgres>,
    pair: Form<EntryHashes>,
) -> Result<Template, Errors> {
    let hashes = pair
        .hashes
        .iter()
        .map(hex::decode)
        .collect::<Result<Vec<Vec<u8>>, _>>()?;
    db.execute(
        "INSERT INTO archive_entries_files (entry, file)
         VALUES ($1, unnest($2::bytea[]))
         ON CONFLICT DO NOTHING",
        &[&pair.entry, &hashes],
    )
    .await?;

    Ok(Template::render(
        "htmx/archive/entry-files",
        context! { id: pair.entry, files: fetch_entry_files(pair.entry, &db).await? },
    ))
}

#[rocket::delete("/archive/detach-files", data = "<pair>")]
pub async fn detach_files(
    _admin: AdminOnly,
    db: Connection<Postgres>,
    pair: Form<EntryHashes>,
) -> Result<Template, Errors> {
    let hashes = pair
        .hashes
        .iter()
        .map(hex::decode)
        .collect::<Result<Vec<Vec<u8>>, _>>()?;
    db.execute(
        "DELETE FROM archive_entries_files
         WHERE entry = $1 AND file = ANY($2::bytea[])",
        &[&pair.entry, &hashes],
    )
    .await?;

    Ok(Template::render(
        "htmx/archive/entry-files",
        context! { id: pair.entry, files: fetch_entry_files(pair.entry, &db).await? },
    ))
}

#[rocket::put("/archive/update-entry-info", data = "<info>")]
pub async fn update_entry_info(
    _admin: AdminOnly,
    db: Connection<Postgres>,
    info: Form<Entry>,
) -> Result<Template, Errors> {
    db.execute(
        "UPDATE archive_entries
                SET title = $2,
                    author = $3,
                    cover = $4,
                    description = $5
                WHERE id = $1",
        &[
            &info.id,
            &info.title,
            &info.author,
            &info.cover,
            &info.description,
        ],
    )
    .await?;
    Ok(Template::render(
        "htmx/archive/entry-info",
        context! {entry: info.into_inner()},
    ))
}

#[rocket::post("/archive/create-entry")]
pub async fn create_entry(_admin: AdminOnly, db: Connection<Postgres>) -> Result<Template, Errors> {
    db.query_one(
        "INSERT INTO archive_entries DEFAULT VALUES RETURNING id, title, author",
        &[],
    )
    .await
    .map(|row| {
        Ok(Entry {
            id: row.try_get(0)?,
            title: row.try_get(1)?,
            author: row.try_get(2)?,
            cover: None,
            description: String::from(""),
            files: vec![],
        })
    })?
    .map(|entry| Template::render("htmx/archive/entry", context! { entry }))
}
