use crate::models::rubrics::Rubric;
use crate::{
    db::{Connection, Postgres},
    errors::Errors,
};
use rocket::serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub async fn get_published(
    db: &Connection<Postgres>,
    cap: Option<i64>,
) -> Result<Vec<ArticleForm>, Errors> {
    if let Some(int) = cap {
        db.query(
            "SELECT id, title, author, cover FROM articles WHERE published
             ORDER BY created_at DESC, title LIMIT $1",
            &[&int],
        )
        .await
    } else {
        db.query(
            "SELECT id, title, author, cover FROM articles WHERE published
             ORDER BY created_at DESC, title",
            &[],
        )
        .await
    }?
    .iter()
    .map(|row| {
        Ok(ArticleForm {
            id: row.try_get(0)?,
            title: row.try_get(1)?,
            author: row.try_get(2)?,
            cover: row.try_get(3)?,
            published: true,
            ..Default::default()
        })
    })
    .collect()
}

pub async fn get_all(db: &Connection<Postgres>) -> Result<Vec<ArticleForm>, Errors> {
    db.query(
        "SELECT id, title, author, cover, published FROM articles ORDER BY created_at DESC, title",
        &[],
    )
    .await?
    .iter()
    .map(|row| {
        Ok(ArticleForm {
            id: row.try_get(0)?,
            title: row.try_get(1)?,
            author: row.try_get(2)?,
            cover: row.try_get(3)?,
            published: row.try_get(4)?,
            ..Default::default()
        })
    })
    .collect()
}

#[derive(Serialize, Deserialize, Default, FromForm)]
#[serde(crate = "rocket::serde")]
pub struct ArticleForm {
    pub id: i32,
    pub title: String,
    pub author: String,
    pub cover: Option<String>,
    pub published: bool,
    pub rubrics: Vec<Rubric>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(crate = "rocket::serde")]
pub struct Article {
    pub id: i32,
    pub title: String,
    pub author: String,
    #[serde(alias = "coverURL")]
    pub cover: Option<String>,
    pub contents: EditorJS,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(crate = "rocket::serde")]
pub struct EditorJS {
    version: String,
    time: i64,
    pub blocks: Vec<Block>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Block {
    id: String,
    #[serde(flatten)]
    pub variant: BlockVariant,
}

#[derive(Serialize, Deserialize)]
#[serde(
    crate = "rocket::serde",
    tag = "type",
    content = "data",
    rename_all = "lowercase"
)]
pub enum BlockVariant {
    Paragraph(Paragraph),
    Heading(Heading),
    List(List),
    Quote(Quote),
    Image(Image),
    Separator(HashMap<i32, i32>),
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Paragraph {
    text: CleanString,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Heading {
    text: CleanString,
    level: i32,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "lowercase")]
pub enum ListType {
    #[serde(rename = "ordered")]
    Ordered,
    #[serde(rename = "unordered")]
    Unordered,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "lowercase")]
pub struct List {
    style: ListType,
    items: Vec<CleanString>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "lowercase")]
pub enum Alignment {
    Left,
    Center,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Quote {
    text: CleanString,
    caption: CleanString,
    alignment: Alignment,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Image {
    pub url: String,
    caption: CleanString,
}

impl Block {
    pub fn empty() -> Self {
        Self {
            id: String::from("1234"),
            variant: BlockVariant::Paragraph(Paragraph {
                text: CleanString(String::new()),
            }),
        }
    }
}

#[derive(Serialize)]
#[serde(from = "String", crate = "rocket::serde")]
pub struct CleanString(String);

impl<'de> Deserialize<'de> for CleanString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: rocket::serde::Deserializer<'de>,
    {
        let str: String = Deserialize::deserialize(deserializer)?;
        Ok(Self(str.replace("&nbsp;", " ")))
    }
}
