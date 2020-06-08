use std::collections::HashMap;

use slug::slugify;

use tokio_postgres::Row;

use crate::error::*;

use crate::auth::*;
use crate::models::*;
use crate::forms::article::*;

use crate::db::*;
use crate::db::util::*;

#[derive(Clone)]
pub struct ArticleService {
  // get one article
  article_by_id: VersionedStatement,
  article_by_slug: VersionedStatement,

  // store article
  store_article: VersionedStatement,
  add_tag: VersionedStatement,
  delete_tag: VersionedStatement,

  // update article
  update_article: VersionedStatement,

  // delete article
  delete_article: VersionedStatement,

  // get multiple articles
  get_articles: VersionedStatement,

  // get user's feed
  get_feed: VersionedStatement,

  // (un)favorite article
  favorite_article: VersionedStatement,
  unfavorite_article: VersionedStatement,
}

lazy_static! {
  static ref ARTICLE_COLUMNS: ColumnMappers = {
    ColumnMappers {
      table_name: "articles",
      columns: vec![
        column("id"),
        column("author_id"),
        column("slug"),
        column("title"),
        column("description"),
        column("body"),
        column("created_at"),
        column("updated_at"),
      ],
    }
  };

  static ref FAVORITE_COLUMNS: ColumnMappers = {
    ColumnMappers {
      table_name: "favorite_articles",
      columns: vec![
        column("user_id"),
        column("article_id"),
      ],
    }
  };
}

fn article_details_from_row(row: &Row) -> ArticleDetails {
  let id: i32 = row.get(0);
  let slug: String = row.get(1);
  let title: String = row.get(2);
  let description: String = row.get(3);
  let body: String = row.get(4);
  let created_at: chrono::NaiveDateTime = row.get(5);
  let updated_at: chrono::NaiveDateTime = row.get(6);
  let tags_list: &str = row.get(7);
  let favorited: i32 = row.get(8);
  let favorites_count: i32 = row.get(9);
  let user_id: i32 = row.get(10);
  let username: String = row.get(11);
  let bio: Option<String> = row.get(12);
  let image: Option<String> = row.get(13);
  let following: i32 = row.get(14);

  ArticleDetails {
    id,
    slug,
    title,
    description,
    body,
    created_at,
    updated_at,
    tag_list: tags_list.split(",").map(|s| s.to_string()).collect(),
    favorited: favorited == 1,
    favorites_count: favorites_count.into(),
    author: Profile {
      user_id,
      username,
      bio,
      image,
      following: following == 1,
    },
  }
}

fn article_details_from_opt_row(row: &Option<Row>) -> Option<ArticleDetails> {
  if let Some(ref row) = row {
    Some(article_details_from_row(row))
  } else {
    None
  }
}

#[derive(Debug)]
enum TagChange {
  Add,
  Remove,
  Keep,
}

static ARTICLE_DETAILS_SELECT: &'static str = r#"
SELECT a.id, slug, title, description, body, a.created_at, a.updated_at,
  (SELECT STRING_AGG(tag_name, ',') FROM article_tags WHERE article_id = a.id) AS TagList,
  (SELECT COUNT(*)::integer FROM favorite_articles WHERE article_id = a.id AND user_id = $1) AS Favorited,
  (SELECT COUNT(*)::integer FROM favorite_articles WHERE article_id = a.id) AS FavoritesCount,
  u.id, u.username, u.bio, u.image,
  (SELECT COUNT(*)::integer FROM followers WHERE user_id = u.id AND follower_id = $1) AS Following
FROM articles a INNER JOIN users u ON a.author_id = u.id
"#;

static FEED_DETAILS_SELECT: &'static str = r#"
WITH following(author_id) AS (
  SELECT user_id FROM followers WHERE follower_id = $1
)
SELECT a.id, slug, title, description, body, a.created_at, a.updated_at,
  (SELECT STRING_AGG(tag_name, ',') FROM article_tags WHERE article_id = a.id) AS TagList,
  (SELECT COUNT(*)::integer FROM favorite_articles WHERE article_id = a.id AND user_id = $1) AS Favorited,
  (SELECT COUNT(*)::integer FROM favorite_articles WHERE article_id = a.id) AS FavoritesCount,
  u.id, u.username, u.bio, u.image,
  1::integer AS Following
FROM following f INNER JOIN articles a ON a.author_id = f.author_id
  INNER JOIN users u ON a.author_id = u.id
"#;

impl ArticleService {
  pub fn new(cl: SharedClient) -> Result<ArticleService> {
    // Build article_by_* queries
    let article_by_id = VersionedStatement::new(cl.clone(),
        &format!(r#"{} WHERE a.id = $2"#, ARTICLE_DETAILS_SELECT))?;
    let article_by_slug = VersionedStatement::new(cl.clone(),
        &format!(r#"{} WHERE a.slug = $2"#, ARTICLE_DETAILS_SELECT))?;

    // store article query
    let store_article = VersionedStatement::new(cl.clone(),
        r#"INSERT INTO articles(author_id, slug, title, description, body)
        VALUES($1, $2, $3, $4, $5) RETURNING id"#)?;
    let add_tag = VersionedStatement::new(cl.clone(),
        r#"INSERT INTO article_tags(article_id, tag_name)
        VALUES($1, $2)"#)?;
    let delete_tag = VersionedStatement::new(cl.clone(),
        r#"DELETE FROM article_tags WHERE article_id = $1 AND tag_name = $2"#)?;

    // update article query
    let update_article = VersionedStatement::new(cl.clone(),
        r#"UPDATE articles SET slug = $2, title = $3, description = $4, body = $5
        WHERE id = $1"#)?;

    // delete article query
    let delete_article = VersionedStatement::new(cl.clone(),
        r#"DELETE FROM articles WHERE id = $1"#)?;

    // Build get_articles queries
    let get_articles = VersionedStatement::new(cl.clone(),
        &format!(r#"{} ORDER BY a.id DESC LIMIT $2 OFFSET $3 "#, ARTICLE_DETAILS_SELECT))?;

    // Build get_feed queries
    let get_feed = VersionedStatement::new(cl.clone(),
        &format!(r#"{} ORDER BY a.id DESC LIMIT $2 OFFSET $3 "#,
        FEED_DETAILS_SELECT))?;

    // (un)favorite
    let favorite_article = VersionedStatement::new(cl.clone(),
        &FAVORITE_COLUMNS.build_upsert("(user_id, article_id)", true))?;
    let unfavorite_article = VersionedStatement::new(cl.clone(),
        "DELETE FROM favorite_articles WHERE user_id = $1 AND article_id = $2")?;

    Ok(ArticleService {
      article_by_id,
      article_by_slug,

      store_article,
      add_tag,
      delete_tag,

      update_article,
      delete_article,

      get_articles,
      get_feed,

      favorite_article,
      unfavorite_article,
    })
  }

  pub async fn prepare(&self) -> Result<()> {
    self.article_by_id.prepare().await?;
    self.article_by_slug.prepare().await?;

    self.store_article.prepare().await?;
    self.add_tag.prepare().await?;
    self.delete_tag.prepare().await?;

    self.update_article.prepare().await?;
    self.delete_article.prepare().await?;

    self.get_articles.prepare().await?;
    self.get_feed.prepare().await?;

    self.favorite_article.prepare().await?;
    self.unfavorite_article.prepare().await?;
    Ok(())
  }

  pub async fn get_by_id(&self, auth: &AuthData, article_id: i32) -> Result<Option<ArticleDetails>> {
    let row = self.article_by_id.query_opt(&[&auth.user_id, &article_id]).await?;
    Ok(article_details_from_opt_row(&row))
  }

  pub async fn get_by_slug(&self, auth: &AuthData, slug: &str) -> Result<Option<ArticleDetails>> {
    let row = self.article_by_slug.query_opt(&[&auth.user_id, &slug]).await?;
    Ok(article_details_from_opt_row(&row))
  }

  pub async fn store(&self, auth: &AuthData, article: &CreateArticle) -> Result<Option<i32>> {
    let slug = slugify(&article.title);
    match self.store_article.query_opt(&[
        &auth.user_id, &slug, &article.title, &article.description, &article.body
      ]).await? {
      Some(row) => {
        let article_id: i32 = row.get(0);
        // add tags to new article.
        for tag in &article.tag_list {
          self.add_tag.execute(&[&article_id, &tag]).await?;
        }
        Ok(Some(article_id))
      },
      None => {
        Ok(None)
      }
    }
  }

  pub async fn update(&self, article: &mut ArticleDetails, req: &UpdateArticle) -> Result<u64> {
    // Update article fields
    if let Some(title) = &req.title {
      article.title = title.clone();
      article.slug = slugify(&title);
    }
    if let Some(desc) = &req.description {
      article.description = desc.clone();
    }
    if let Some(body) = &req.body {
      article.body = body.clone();
    }
    // store article changes.
    self.update_article.execute(&[
        &article.id, &article.slug, &article.title, &article.description, &article.body
    ]).await?;

    // update list of tags.
    let mut tags = HashMap::new();
    for tag in &article.tag_list {
      // mark all old tags as remove.
      tags.insert(tag, TagChange::Remove);
    }
    for tag in &req.tag_list {
      tags.entry(&tag)
        .and_modify(|e| *e = TagChange::Keep)
        .or_insert(TagChange::Add);
    }

    // apply tag changes
    for (tag, change) in tags.iter() {
      match change {
        TagChange::Add => {
          self.add_tag.execute(&[&article.id, &tag]).await?;
        },
        TagChange::Remove => {
          self.delete_tag.execute(&[&article.id, &tag]).await?;
        },
        TagChange::Keep => (),
      }
    }

    Ok(1)
  }

  pub async fn delete(&self, article_id: i32) -> Result<u64> {
    Ok(self.delete_article.execute(&[&article_id]).await?)
  }

  pub async fn favorite(&self, auth: &AuthData, article_id: i32) -> Result<u64> {
    Ok(self.favorite_article.execute(&[&auth.user_id, &article_id]).await?)
  }

  pub async fn unfavorite(&self, auth: &AuthData, article_id: i32) -> Result<u64> {
    Ok(self.unfavorite_article.execute(&[&auth.user_id, &article_id]).await?)
  }

  pub async fn get_articles(&self, auth: &AuthData, req: ArticleRequest) -> Result<Vec<ArticleDetails>> {
    let limit = req.limit.unwrap_or(20);
    let offset = req.offset.unwrap_or(0);
    let rows = self.get_articles.query(&[&auth.user_id, &limit, &offset]).await?;
    Ok(rows.iter().map(article_details_from_row).collect())
  }

  pub async fn get_feed(&self, auth: &AuthData, req: FeedRequest) -> Result<Vec<ArticleDetails>> {
    let user_id = auth.user_id;
    let limit = req.limit.unwrap_or(20);
    let offset = req.offset.unwrap_or(0);
    let rows = self.get_feed.query(&[&user_id, &limit, &offset]).await?;
    Ok(rows.iter().map(article_details_from_row).collect())
  }
}
