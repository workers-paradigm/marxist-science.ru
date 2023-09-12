use std::path::{Path, PathBuf};

use rocket::{
    form::Form,
    fs::relative,
    response::Redirect,
    serde::{json::Json, Deserialize, Serialize},
};

use rocket_dyn_templates::{context, Template};

use crate::auth::AdminOnly;
use crate::db::{Connection, Postgres, Transaction};
use crate::errors::Errors;
use crate::models::{FileRecord, HashedFileBuf};

#[derive(FromForm)]
pub struct FilesHolder {
    entry: Option<i32>,
    ext: Vec<String>,
    files: Vec<HashedFileBuf>,
}

async fn exists(db: &Transaction<'_>, hash: &[u8]) -> Result<bool, Errors> {
    db.query_one("SELECT EXISTS(SELECT 1 FROM files WHERE id = $1)", &[&hash])
        .await?
        .try_get::<usize, bool>(0)
        .map_err(Into::into)
}

fn relative_path_to(hash: &str, ext: &String) -> PathBuf {
    Path::new(relative!["/static/uploads/"]).join(hash.to_owned() + "." + ext)
}

pub fn path_to(hash: &str, ext: &String) -> PathBuf {
    Path::new("/static/uploads/").join(hash.to_owned() + "." + ext)
}

pub async fn upload_file(db: &Transaction<'_>, file: &mut HashedFileBuf) -> Result<String, Errors>
where
{
    let hash: &[u8] = &file.hash[..];
    let hex = hex::encode(hash);
    let ext = file.content_type.extension_err()?;
    let location = relative_path_to(&hex, &ext);
    if !(exists(&db, hash).await?) {
        file.persist_to(&location)?;
        db.execute(
            "INSERT INTO files (id, title, ext) VALUES ($1, $2, $3)",
            &[&&file.hash[..], &file.name, &ext],
        )
        .await?;
    }

    Ok(location.display().to_string())
}

#[rocket::put("/upload/one_file", data = "<file>")]
pub async fn upload_one(
    _admin: AdminOnly,
    mut db: Connection<Postgres>,
    mut file: Form<HashedFileBuf>,
) -> Result<Json<FileRecord>, Errors> {
    let db = db.transaction().await?;
    upload_file(&db, &mut file).await?;
    let ext = file.content_type.extension_err()?;
    db.commit().await?;
    Ok(Json(FileRecord::from(
        &file.hash[..],
        file.name.clone(),
        ext,
    )))
}

#[rocket::put("/upload/files", data = "<files_holder>")]
pub async fn files(
    _admin: AdminOnly,
    mut db: Connection<Postgres>,
    mut files_holder: Form<FilesHolder>,
) -> Result<Redirect, Errors> {
    let mut output = vec![];
    let db = db.transaction().await?;
    for file in &mut files_holder.files {
        upload_file(&db, file).await?;
        let ext = file.content_type.extension_err()?;
        output.push(FileRecord::from(&file.hash[..], file.name.clone(), ext));
    }

    db.commit().await?;
    let id = if let Some(int) = files_holder.entry {
        "id=".to_owned() + &int.to_string()
    } else {
        String::new()
    };
    let ext = files_holder
        .into_inner()
        .ext
        .iter()
        .map(|extension| format!("&ext[]={}", extension))
        .reduce(|acc, right| acc + &right)
        .unwrap_or(String::new());
    Ok(Redirect::to("/upload/file-picker?".to_owned() + &id + &ext))
}

#[rocket::get("/upload/file-picker?<id>&<ext>")]
pub async fn get(
    db: Connection<Postgres>,
    id: Option<i32>,
    ext: Vec<String>,
) -> Result<Template, Errors> {
    db.query(
        "SELECT id, title, ext FROM files WHERE ext LIKE ANY($1)",
        &[&ext],
    )
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
    .map(|vec| Template::render("htmx/archive/file-picker", context! { id, ext, files: vec }))
}

#[derive(Debug, Serialize, Deserialize, FromForm)]
#[serde(crate = "rocket::serde")]
pub struct Hashes {
    entry: Option<i32>,
    hashes: Vec<String>,
    ext: Vec<String>,
}
#[rocket::delete("/upload/delete", data = "<files>")]
pub async fn delete(
    _admin: AdminOnly,
    db: Connection<Postgres>,
    files: Form<Hashes>,
) -> Result<Redirect, Errors> {
    use std::fs;
    let hashes = files
        .hashes
        .iter()
        .map(hex::decode)
        .collect::<Result<Vec<Vec<u8>>, _>>()?;
    db.query(
        "DELETE FROM files WHERE id = ANY($1::bytea[]) RETURNING id, ext",
        &[&hashes],
    )
    .await?
    .iter()
    .map(|row| {
        fs::remove_file(relative_path_to(
            &hex::encode(row.try_get::<usize, Vec<u8>>(0)?),
            &row.try_get(1)?,
        ))
        .map_err(Into::into)
    })
    .collect::<Result<_, Errors>>()?;

    let id = if let Some(int) = files.entry {
        "id=".to_owned() + &int.to_string()
    } else {
        String::new()
    };
    let ext = files
        .into_inner()
        .ext
        .iter()
        .map(|extension| format!("&ext[]={}", extension))
        .reduce(|acc, right| acc + &right)
        .unwrap_or(String::new());
    Ok(Redirect::to("/upload/file-picker?".to_owned() + &id + &ext))
}
