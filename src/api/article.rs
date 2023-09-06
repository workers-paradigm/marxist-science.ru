use rocket::form::Form;
use rocket::serde::json::{self, Json};
use rocket_dyn_templates::{context, Template};

use crate::auth::AdminOnly;
use crate::db::{Connection, Postgres};
use crate::errors::Errors;
use crate::models::{
    article::{ArticleForm, BlockVariant, EditorJS},
    Article, Block, Preview,
};

#[rocket::get("/articles/contents/<id>")]
pub async fn contents(id: i32, db: Connection<Postgres>) -> Result<Json<String>, Errors> {
    db.query_opt("SELECT contents FROM articles WHERE id = $1", &[&id])
        .await?
        .map_or(
            Err(Errors::NotFound),
            |r| Ok(r.try_get::<usize, String>(0)?),
        )
        .map(Json)
}

#[rocket::post("/articles/create")]
pub async fn create(_admin: AdminOnly, db: Connection<Postgres>) -> Result<Template, Errors> {
    let title = "Черновик";
    let row = db
        .query_one(
            "INSERT INTO articles (title) VALUES ($1) RETURNING id",
            &[&title],
        )
        .await?;

    let article_info = ArticleForm {
        id: row.try_get(0)?,
        title: title.to_owned(),
        cover: None,
        published: false,
    };
    Ok(Template::render(
        "htmx/articles/entry",
        context! { entry: article_info },
    ))
}

#[rocket::get("/articles/edit/<id>")]
pub async fn edit(
    _admin: AdminOnly,
    id: i32,
    db: Connection<Postgres>,
) -> Result<Template, Errors> {
    db.query_opt("SELECT title FROM articles WHERE id = $1", &[&id])
        .await?
        .map_or(Err(Errors::NotFound), |row| {
            // -> Result<Article, Errors>
            Ok(Article {
                id,
                title: row.try_get(0)?,
                ..Default::default()
            })
        })
        .map(|article| Template::render("articles/edit", article))
}

#[rocket::put("/articles/save", data = "<article>")]
pub async fn save(
    _admin: AdminOnly,
    db: Connection<Postgres>,
    mut article: Json<Article>,
) -> Result<(), Errors> {
    // https://github.com/codex-team/editor.js/pull/2454
    // this is needed as a workaround because EditorJS can and will crash on your ass if there isn't at least one block in the contents json.
    if article.contents.blocks.is_empty() {
        article.contents.blocks.push(Block::empty());
    }
    db.execute(
        "UPDATE articles SET title = $2, contents = $3, cover = $4 WHERE id = $1",
        &[
            &article.id,
            &article.title,
            &json::to_string(&article.contents)?,
            &article.cover,
        ],
    )
    .await
    .map(|_| ())
    .map_err(Into::into)
}

#[rocket::get("/articles/view/<id>")]
pub async fn view(id: i32, db: Connection<Postgres>) -> Result<Template, Errors> {
    db.query_opt(
        "SELECT title, cover, contents FROM articles WHERE id = $1 AND published",
        &[&id],
    )
    .await?
    .map_or(Err(Errors::NotFound), |row| {
        let title: String = row.try_get(0)?;
        let cover: Option<String> = row.try_get(1)?;
        let mut contents: EditorJS = json::from_str(row.try_get::<usize, &str>(2)?)?;
        if let Some(url) = &cover {
            contents.blocks.retain(|block| match &block.variant {
                BlockVariant::Image(img) => img.url != url.as_str(),
                _ => true,
            });
        }
        Ok(Article {
            id,
            title,
            cover,
            contents,
        })
    })
    .map(|article| Template::render("articles/view", context! { article }))
}

#[rocket::delete("/articles/delete", data = "<article>")]
pub async fn delete(
    _admin: AdminOnly,
    article: Form<ArticleForm>,
    db: Connection<Postgres>,
) -> Result<(), Errors> {
    db.execute("DELETE FROM articles WHERE id = $1", &[&article.id])
        .await
        .map(|_| ())
        .map_err(Into::into)
}

#[rocket::put("/articles/save-info", data = "<article>")]
pub async fn save_info(
    _admin: AdminOnly,
    db: Connection<Postgres>,
    article: Form<ArticleForm>,
) -> Result<Template, Errors> {
    db.execute(
        "UPDATE articles SET title = $2, published = $3 WHERE id = $1",
        &[&article.id, &article.title, &article.published],
    )
    .await
    .map(|_| {
        Template::render(
            "htmx/articles/entry",
            context! { entry: article.into_inner(), just_saved: true },
        )
    })
    .map_err(|e| e.into())
}

#[rocket::get("/articles")]
pub async fn index(db: Connection<Postgres>) -> Result<Template, Errors> {
    fetch_published_previews(&db)
        .await
        .map(|previews| Template::render("articles/index", context! { previews: previews }))
}

pub async fn fetch_published_previews(db: &Connection<Postgres>) -> Result<Vec<Preview>, Errors> {
    db.query(
        "SELECT id, title, cover FROM articles WHERE published
         ORDER BY created_at DESC, title",
        &[],
    )
    .await?
    .iter()
    .map(|row| {
        Ok(Preview {
            id: row.try_get(0)?,
            title: row.try_get(1)?,
            cover: row.try_get(2)?,
            published: true,
        })
    })
    .collect()
}

pub async fn fetch_all_previews(db: &Connection<Postgres>) -> Result<Vec<Preview>, Errors> {
    db.query(
        "SELECT id, title, cover, published FROM articles ORDER BY created_at DESC, title",
        &[],
    )
    .await?
    .iter()
    .map(|row| {
        Ok(Preview {
            id: row.try_get(0)?,
            title: row.try_get(1)?,
            cover: row.try_get(2)?,
            published: row.try_get(3)?,
        })
    })
    .collect()
}
