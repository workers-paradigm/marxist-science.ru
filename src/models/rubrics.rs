use crate::{
    db::{Connection, Postgres},
    errors::Errors,
};
use rocket::serde::{Deserialize, Serialize};

pub async fn get_all(db: &Connection<Postgres>) -> Result<Vec<Rubric>, Errors> {
    db.query("SELECT id, title, cover FROM rubrics", &[])
        .await?
        .iter()
        .map(|row| {
            Ok(Rubric {
                id: row.try_get(0)?,
                title: row.try_get(1)?,
                cover: row.try_get(2)?,
            })
        })
        .collect::<Result<_, _>>()
}

pub async fn get_populated(
    db: &Connection<Postgres>,
    cap: Option<i64>,
) -> Result<Vec<Rubric>, Errors> {
    let result = if let Some(int) = cap {
        db.query("SELECT DISTINCT r.id, r.title, r.cover FROM rubrics as r JOIN articles_rubrics as ar ON r.id = ar.rubric LIMIT $1", &[&int]).await
    } else {
        db.query("SELECT DISTINCT r.id, r.title, r.cover FROM rubrics as r JOIN articles_rubrics as ar ON r.id = ar.rubric", &[]).await
    };
    result?
        .iter()
        .map(|row| {
            Ok(Rubric {
                id: row.try_get(0)?,
                title: row.try_get(1)?,
                cover: row.try_get(2)?,
            })
        })
        .collect::<Result<_, _>>()
}

#[derive(Serialize, Deserialize, FromForm)]
#[serde(crate = "rocket::serde")]
pub struct Rubric {
    pub id: i32,
    pub title: String,
    pub cover: Option<String>,
}
