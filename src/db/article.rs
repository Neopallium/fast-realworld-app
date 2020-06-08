use crate::error::*;

use crate::auth::*;
use crate::models::*;
use crate::forms::article::*;

use crate::db::*;
use crate::db::util::*;

use tokio_postgres::Row;

#[derive(Clone)]
pub struct ArticleService {
  // get one article
  article_by_slug: VersionedStatement,

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
    let article_by_slug = VersionedStatement::new(cl.clone(),
        &format!(r#"{} WHERE a.slug = $2"#, ARTICLE_DETAILS_SELECT))?;

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
      article_by_slug,

      get_articles,
      get_feed,

      favorite_article,
      unfavorite_article,
    })
  }

  pub async fn prepare(&self) -> Result<()> {
    self.article_by_slug.prepare().await?;

    self.get_articles.prepare().await?;
    self.get_feed.prepare().await?;

    self.favorite_article.prepare().await?;
    self.unfavorite_article.prepare().await?;
    Ok(())
  }

  pub async fn get_by_slug(&self, auth: &AuthData, slug: &str) -> Result<Option<ArticleDetails>> {
    let row = self.article_by_slug.query_opt(&[&auth.user_id, &slug]).await?;
    Ok(article_details_from_opt_row(&row))
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
