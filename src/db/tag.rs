use crate::error::*;

use crate::models::*;
use crate::forms::tag::*;

use crate::db::*;
use crate::db::util::*;

#[derive(Clone)]
pub struct TagService {
  // get multiple tags
  get_tags: VersionedStatement,
}

lazy_static! {
  static ref ARTICLE_COLUMNS: ColumnMappers = {
    ColumnMappers {
      table_name: "tags",
      columns: vec![
        column("article_id"),
        column("tag_name"),
        column("created_at"),
        column("updated_at"),
      ],
    }
  };
}

impl TagService {
  pub fn new(cl: SharedClient) -> Result<TagService> {
    // Build get_tags queries
    let get_tags = VersionedStatement::new(cl.clone(),
        r#"SELECT tag_name FROM article_tags GROUP BY tag_name ORDER BY tag_name"#)?;

    Ok(TagService {
      get_tags,
    })
  }

  pub async fn prepare(&self) -> Result<()> {
    self.get_tags.prepare().await?;
    Ok(())
  }

  pub async fn get_tags(&self) -> Result<TagList> {
    let rows = self.get_tags.query(&[]).await?;
    Ok(TagList{
      tags: rows.iter().map(|r| TagName(r.get(0))).collect(),
    })
  }
}
