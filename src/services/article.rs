use log::*;

use actix_web::{
  get, post, delete, web, HttpResponse,
  Error
};

use crate::error::*;
use crate::app::*;
use crate::models::*;
use crate::forms::article::*;
use crate::db::DbService;

/// Get list of articles
#[get("/articles")]
async fn list(
  db: web::Data<DbService>,
  req: web::Query<ArticleRequest>
) -> Result<HttpResponse, Error> {

  // TODO: auth
  // TODO: author, tag, favorited filters.
  let articles = db.article.get_articles(None, req.into_inner()).await?;

  Ok(HttpResponse::Ok().json(ArticleList::<ArticleDetails> {
    articles_count: articles.len(),
    articles,
  }))
}

/// get article by slug
#[get("/articles/{slug}")]
async fn get_article(
  db: web::Data<DbService>,
  slug: web::Path<String>,
) -> Result<HttpResponse, Error> {
  // TODO: auth

  if let Some(article) = db.article.get_by_slug(None, &slug).await? {
    Ok(HttpResponse::Ok().json(ArticleOut::<ArticleDetails> {
      article,
    }))
  } else {
    Ok(HttpResponse::NotFound().finish())
  }
}

/// post new article
#[post("/articles")]
async fn store_article(
  _db: web::Data<DbService>,
  article: web::Json<ArticleOut<CreateArticle>>,
) -> Result<HttpResponse, Error> {
  let article = article.into_inner().article;

  info!("Article - new article: TODO");
  Ok(HttpResponse::Ok().json(article))
}

/// post update to existing article
#[post("/articles/{slug}")]
async fn update_article(
  cfg: web::Data<ArticleService>,
  _db: web::Data<DbService>,
  _slug: web::Path<String>,
  article: web::Json<ArticleOut<UpdateArticle>>,
) -> Result<HttpResponse, Error> {
  let article = article.into_inner().article;

  if cfg.allow_update {
    info!("Article - update article: TODO");
  }
  Ok(HttpResponse::Ok().json(article))
}

/// delete an existing article
#[delete("/articles/{slug}")]
async fn delete_article(
  cfg: web::Data<ArticleService>,
  _db: web::Data<DbService>,
  _slug: web::Path<String>,
) -> Result<HttpResponse, Error> {
  if cfg.allow_delete {
    info!("Article - new article: TODO");
  }
  Ok(HttpResponse::Ok().finish())
}

#[derive(Debug, Clone, Default)]
pub struct ArticleService {
  pub allow_update: bool,
  pub allow_delete: bool,
}

impl super::Service for ArticleService {
  fn load_app_config(&mut self, config: &AppConfig, _prefix: &str) -> Result<()> {
    self.allow_update = config.get_bool("Article.allow_update")?.unwrap_or(false);
    self.allow_delete = config.get_bool("Article.allow_delete")?.unwrap_or(false);
    Ok(())
  }

  fn api_config(&self, web: &mut web::ServiceConfig) {
    web
      .data(self.clone())
      .service(list)
      .service(get_article)
      .service(store_article)
      .service(update_article)
      .service(delete_article);
  }
}

pub fn new_factory() -> ArticleService {
  Default::default()
}
