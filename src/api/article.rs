use rocket::form::Form;
use rocket::serde::json::{self, Json};
use rocket_dyn_templates::{context, Template};

use crate::auth::AdminOnly;
use crate::db::{Connection, Postgres};
use crate::errors::Errors;
use crate::models::{
    article::{self, ArticleForm, EditorJS},
    rubrics::{self, Rubric},
    Article, Block,
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
        ..Default::default()
    };
    Ok(Template::render(
        "htmx/articles/entry",
        context! { entry: article_info },
    ))
}

pub async fn get_article_rubrics(
    db: &Connection<Postgres>,
    id: i32,
) -> Result<Vec<Rubric>, Errors> {
    db.query(
        "SELECT rubrics.id, rubrics.title, rubrics.cover FROM rubrics JOIN articles_rubrics ON articles_rubrics.rubric = rubrics.id JOIN articles ON articles.id = articles_rubrics.article WHERE articles.id = $1",
        &[&id])
        .await?
        .iter()
        .map(|row| Ok(Rubric {
            id: row.try_get(0)?,
            title: row.try_get(1)?,
            cover: row.try_get(2)?,
        })).collect::<Result<Vec<_>, Errors>>()
}

#[rocket::get("/articles/edit/<id>")]
pub async fn edit(
    _admin: AdminOnly,
    id: i32,
    db: Connection<Postgres>,
) -> Result<Template, Errors> {
    let mut article = db
        .query_opt("SELECT title, author FROM articles WHERE id = $1", &[&id])
        .await?
        .map_or(Err(Errors::NotFound), |row| {
            // -> Result<Article, Errors>
            Ok(ArticleForm {
                id,
                title: row.try_get(0)?,
                author: row.try_get(1)?,
                ..Default::default()
            })
        })?;
    Ok(Template::render("articles/edit", article))
}

#[rocket::get("/articles/rubrics-of-article/<id>")]
pub async fn rubrics_of_article(
    _admin: AdminOnly,
    db: Connection<Postgres>,
    id: i32,
) -> Result<Template, Errors> {
    get_article_rubrics(&db, id)
        .await
        .map(|rubrics| Template::render("articles/rubrics-of-article", context! { rubrics, id }))
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
        "UPDATE articles SET title = $2, author = $3, contents = $4, cover = $5 WHERE id = $1",
        &[
            &article.id,
            &article.title,
            &article.author,
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
        "SELECT title, author, cover, contents FROM articles WHERE id = $1 AND published",
        &[&id],
    )
    .await?
    .map_or(Err(Errors::NotFound), |row| {
        let title: String = row.try_get(0)?;
        let author: String = row.try_get(1)?;
        let cover: Option<String> = row.try_get(2)?;
        let contents: EditorJS = json::from_str(row.try_get(3)?)?;
        Ok(Article {
            id,
            title,
            author,
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
        "UPDATE articles SET title = $2, author = $3, published = $4 WHERE id = $1",
        &[
            &article.id,
            &article.title,
            &article.author,
            &article.published,
        ],
    )
    .await?;
    Ok(Template::render(
        "htmx/articles/entry",
        context! { entry: article::get_one(&db, article.id).await? },
    ))
}

#[rocket::get("/articles")]
pub async fn index(db: Connection<Postgres>) -> Result<Template, Errors> {
    let articles = article::get_published(&db, None).await?;
    let rubrics = rubrics::get_populated(&db, None).await?;
    Ok(Template::render(
        "articles/index",
        context! { rubrics, articles },
    ))
}
