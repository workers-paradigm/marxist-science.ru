use rocket::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, FromForm)]
#[serde(crate = "rocket::serde")]
pub struct ArticleForm {
    pub id: i32,
    pub title: String,
    pub cover: Option<String>,
    pub published: bool,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(crate = "rocket::serde")]
pub struct Preview {
    pub id: i32,
    pub title: String,
    pub cover: Option<String>,
    pub published: bool,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(crate = "rocket::serde")]
pub struct Article {
    pub id: i32,
    pub title: String,
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
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Paragraph {
    text: String,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Heading {
    text: String,
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
    items: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "lowercase")]
pub enum Alignment {
    Left,
    Right,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Quote {
    text: String,
    caption: String,
    alignment: Alignment,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Image {
    pub url: String,
    caption: String,
}

impl Block {
    pub fn empty() -> Self {
        Self {
            id: String::from("1234"),
            variant: BlockVariant::Paragraph(Paragraph {
                text: String::new(),
            }),
        }
    }
}
