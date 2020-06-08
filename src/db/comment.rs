use crate::error::*;

use crate::auth::*;
use crate::models::*;
use crate::forms::*;

use crate::db::*;
use crate::db::util::*;

use tokio_postgres::Row;

#[derive(Clone)]
pub struct CommentService {
  // get comment
  comment_by_id: VersionedStatement,

  // store comment
  store_comment: VersionedStatement,

  // delete comment
  delete_comment: VersionedStatement,

  // get multiple comments
  comments_by_slug: VersionedStatement,
}

lazy_static! {
  static ref COMMENT_COLUMNS: ColumnMappers = {
    ColumnMappers {
      table_name: "comments",
      columns: vec![
        column("id"),
        column("article_id"),
        column("user_id"),
        column("body"),
        column("created_at"),
        column("updated_at"),
      ],
    }
  };
}

fn comment_details_from_row(row: &Row) -> CommentDetails {
  let id: i32 = row.get(0);
  let body: String = row.get(1);
  let created_at: chrono::NaiveDateTime = row.get(2);
  let updated_at: chrono::NaiveDateTime = row.get(3);
  let user_id: i32 = row.get(4);
  let username: String = row.get(5);
  let bio: Option<String> = row.get(6);
  let image: Option<String> = row.get(7);
  let following: i32 = row.get(8);

  CommentDetails {
    id,
    created_at,
    updated_at,
    body,
    author: Profile {
      user_id,
      username,
      bio,
      image,
      following: following == 1,
    },
  }
}

fn comment_details_from_opt_row(row: &Option<Row>) -> Option<CommentDetails> {
  if let Some(ref row) = row {
    Some(comment_details_from_row(row))
  } else {
    None
  }
}

static COMMENT_DETAILS_SELECT: &'static str = r#"
SELECT c.id, c.body, c.created_at, c.updated_at,
  u.id, u.username, u.bio, u.image,
  (SELECT COUNT(*)::integer FROM followers WHERE user_id = u.id AND follower_id = $1) AS Following
FROM comments c INNER JOIN users u ON c.user_id = u.id
"#;

impl CommentService {
  pub fn new(cl: SharedClient) -> Result<CommentService> {
    // Build get_comment_* queries
    let comment_by_id = VersionedStatement::new(cl.clone(),
        &format!(r#"{} WHERE c.id = $2"#, COMMENT_DETAILS_SELECT))?;

    // insert comment query
    let store_comment = VersionedStatement::new(cl.clone(),
        r#"INSERT INTO comments(article_id, user_id, body)
        VALUES($1, $2, $3) RETURNING id"#)?;

    // delete comment query
    let delete_comment = VersionedStatement::new(cl.clone(),
        r#"DELETE FROM comments WHERE id = $1"#)?;

    // Build get_comments_* queries
    let comments_by_slug = VersionedStatement::new(cl.clone(),
        &format!(r#"{} INNER JOIN articles a ON c.article_id = a.id
          WHERE a.slug = $2
          ORDER BY c.id DESC"#, COMMENT_DETAILS_SELECT))?;

    Ok(CommentService {
      comment_by_id,

      store_comment,
      delete_comment,

      comments_by_slug,
    })
  }

  pub async fn prepare(&self) -> Result<()> {
    self.comment_by_id.prepare().await?;

    self.store_comment.prepare().await?;
    self.delete_comment.prepare().await?;

    self.comments_by_slug.prepare().await?;

    Ok(())
  }

  pub async fn get_comment_by_id(&self, auth: &AuthData, comment_id: i32) -> Result<Option<CommentDetails>> {
    let row = self.comment_by_id.query_opt(&[&auth.user_id, &comment_id]).await?;
    Ok(comment_details_from_opt_row(&row))
  }

  pub async fn store(&self, auth: &AuthData, article_id: i32, comment: &CreateComment) -> Result<Option<i32>> {
    Ok(self.store_comment.query_opt(&[&article_id, &auth.user_id, &comment.body])
      .await?.map(|row| row.get(0))
    )
  }

  pub async fn delete(&self, comment_id: i32) -> Result<u64> {
    Ok(self.delete_comment.execute(&[&comment_id]).await?)
  }

  pub async fn get_comments_by_slug(&self, auth: &AuthData, slug: &str) -> Result<Vec<CommentDetails>> {
    let rows = self.comments_by_slug.query(&[&auth.user_id, &slug]).await?;
    Ok(rows.iter().map(comment_details_from_row).collect())
  }
}
