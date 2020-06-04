use crate::error::*;

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
}

fn article_details_from_row(row: &Row) -> ArticleDetails {
  let slug: String = row.get(0);
  let title: String = row.get(1);
  let description: String = row.get(2);
  let body: String = row.get(3);
  let created_at: chrono::NaiveDateTime = row.get(4);
  let updated_at: chrono::NaiveDateTime = row.get(5);
  let tags_list: &str = row.get(6);
  let favorited: i64 = row.get(7);
  let favorites_count: i64 = row.get(8);
  let username: String = row.get(9);
  let bio: Option<String> = row.get(10);
  let image: Option<String> = row.get(11);
  let following: i64 = row.get(12);

  ArticleDetails {
    slug,
    title,
    description,
    body,
    created_at,
    updated_at,
    tag_list: tags_list.split(",").map(|s| s.to_string()).collect(),
    favorited: favorited == 1,
    favorites_count,
    author: Profile {
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
SELECT slug, title, description, body, a.created_at, a.updated_at,
  (SELECT STRING_AGG(tag_name, ',') FROM article_tags WHERE article_id = a.id) AS TagList,
  (SELECT COUNT(*) FROM favorite_articles WHERE article_id = a.id AND user_id = $1) AS Favorited,
  (SELECT COUNT(*) FROM favorite_articles WHERE article_id = a.id) AS FavoritesCount,
  u.username, u.bio, u.image,
  (SELECT COUNT(*) FROM followers WHERE user_id = u.id AND follower_id = $1) AS Following
FROM articles a INNER JOIN users u ON a.author_id = u.id
"#;

impl ArticleService {
  pub fn new(cl: SharedClient) -> Result<ArticleService> {
    // Build article_by_* queries
    let article_by_slug = VersionedStatement::new(cl.clone(),
        &format!(r#"{} WHERE a.slug = $2"#, ARTICLE_DETAILS_SELECT))?;

    // Build get_articles queries
    let get_articles = VersionedStatement::new(cl.clone(),
        &format!(r#"{} ORDER BY a.id DESC LIMIT $2 OFFSET $3 "#, ARTICLE_DETAILS_SELECT))?;

    Ok(ArticleService {
      article_by_slug,
      get_articles,
    })
  }

  pub async fn prepare(&self) -> Result<()> {
    self.article_by_slug.prepare().await?;

    Ok(())
  }

  pub async fn get_by_slug(&self, user_id: Option<i32>, slug: &str) -> Result<Option<ArticleDetails>> {
    let user_id = user_id.unwrap_or(0);
    let row = self.article_by_slug.query_opt(&[&user_id, &slug]).await?;
    Ok(article_details_from_opt_row(&row))
  }

  pub async fn get_articles(&self, user_id: Option<i32>, req: ArticleRequest) -> Result<Vec<ArticleDetails>> {
    let user_id = user_id.unwrap_or(0);
    let limit = req.limit.unwrap_or(20);
    let offset = req.offset.unwrap_or(0);
    let rows = self.get_articles.query(&[&user_id, &limit, &offset]).await?;
    Ok(rows.iter().map(article_details_from_row).collect())
  }
}
