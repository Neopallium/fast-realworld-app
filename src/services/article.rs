use actix_web::{
  get, post, put, delete, web, HttpResponse,
  Error
};

use crate::error::*;
use crate::app::*;

use crate::models::*;
use crate::forms::*;

use crate::db::DbService;

use crate::auth::AuthData;
use crate::middleware::Auth;

/// Get list of articles
#[get("/articles", wrap="Auth::optional()")]
async fn list(
  auth: Option<AuthData>,
  db: web::Data<DbService>,
  req: web::Query<ArticleRequest>
) -> Result<HttpResponse, Error> {
  let auth = auth.unwrap_or_default();

  // TODO: author, tag, favorited filters.
  let articles = db.article.get_articles(&auth, req.into_inner()).await?;

  Ok(HttpResponse::Ok().json(ArticleList::<ArticleDetails> {
    articles_count: articles.len(),
    articles,
  }))
}

/// Get current user's feed
#[get("/articles/feed", wrap="Auth::required()")]
async fn feed(
  auth: AuthData,
  db: web::Data<DbService>,
  req: web::Query<FeedRequest>
) -> Result<HttpResponse, Error> {

  let articles = db.article.get_feed(&auth, req.into_inner()).await?;

  Ok(HttpResponse::Ok().json(ArticleList::<ArticleDetails> {
    articles_count: articles.len(),
    articles,
  }))
}

/// get article by slug
#[get("/articles/{slug}", wrap="Auth::optional()")]
async fn get_article(
  auth: Option<AuthData>,
  db: web::Data<DbService>,
  slug: web::Path<String>,
) -> Result<HttpResponse, Error> {
  let auth = auth.unwrap_or_default();

  match db.article.get_by_slug(&auth, &slug).await? {
    Some(article) => {
      Ok(HttpResponse::Ok().json(ArticleOut::<ArticleDetails> {
        article,
      }))
    },
    None => {
      Ok(HttpResponse::NotFound().json(json!({
        "error": "Article not found",
      })))
    }
  }
}

/// post new article
#[post("/articles", wrap="Auth::required()")]
async fn store_article(
  auth: AuthData,
  db: web::Data<DbService>,
  req: web::Json<ArticleOut<CreateArticle>>,
) -> Result<HttpResponse, Error> {
  match db.article.store(&auth, &req.article).await? {
    Some(article_id) => {
      match db.article.get_by_id(&auth, article_id).await? {
        Some(article) => {
          Ok(HttpResponse::Ok().json(ArticleOut::<ArticleDetails> {
            article,
          }))
        },
        None => {
          Ok(HttpResponse::NotFound().json(json!({
            "error": "Failed to find saved article.",
          })))
        }
      }
    },
    None => {
      Ok(HttpResponse::InternalServerError().json(json!({
        "error": "Failed to store article",
      })))
    }
  }
}

/// post update to existing article
#[put("/articles/{slug}", wrap="Auth::required()")]
async fn update_article(
  auth: AuthData,
  cfg: web::Data<ArticleService>,
  db: web::Data<DbService>,
  slug: web::Path<String>,
  req: web::Json<ArticleOut<UpdateArticle>>,
) -> Result<HttpResponse, Error> {
  match db.article.get_by_slug(&auth, &slug).await? {
    Some(mut article) => {
      if cfg.allow_update && article.author.user_id == auth.user_id {
        let old_article = article.clone();
        let article = if db.article.update(&mut article, &req.article).await? > 0 {
          // article updated return updated article.
          article
        } else {
          // Failed to update article, return old article.
          old_article
        };
        Ok(HttpResponse::Ok().json(ArticleOut::<ArticleDetails> {
          article,
        }))
      } else {
        Ok(HttpResponse::Forbidden().json(json!({
          "error": "Update article disabled.",
        })))
      }
    },
    None => {
      Ok(HttpResponse::NotFound().json(json!({
        "error": "Article not found",
      })))
    }
  }
}

/// delete an existing article
#[delete("/articles/{slug}", wrap="Auth::required()")]
async fn delete_article(
  auth: AuthData,
  cfg: web::Data<ArticleService>,
  db: web::Data<DbService>,
  slug: web::Path<String>,
) -> Result<HttpResponse, Error> {
  match db.article.get_by_slug(&auth, &slug).await? {
    Some(article) => {
      if cfg.allow_delete && article.author.user_id == auth.user_id {
        db.article.delete(article.id).await?;
        Ok(HttpResponse::Ok().finish())
      } else {
        Ok(HttpResponse::Forbidden().json(json!({
          "error": "Delete article disabled.",
        })))
      }
    },
    None => {
      Ok(HttpResponse::NotFound().json(json!({
        "error": "Article not found",
      })))
    }
  }
}

/////////////////////////////// Article Comments

/// get article comments by slug
#[get("/articles/{slug}/comments", wrap="Auth::optional()")]
async fn get_comments(
  auth: Option<AuthData>,
  db: web::Data<DbService>,
  slug: web::Path<String>,
) -> Result<HttpResponse, Error> {
  let auth = auth.unwrap_or_default();

  let comments = db.comment.get_comments_by_slug(&auth, &slug).await?;
  Ok(HttpResponse::Ok().json(CommentList {
    comments,
  }))
}

/// Add comment to article
#[post("/articles/{slug}/comments", wrap="Auth::required()")]
async fn store_comment(
  auth: AuthData,
  cfg: web::Data<ArticleService>,
  db: web::Data<DbService>,
  slug: web::Path<String>,
  req: web::Json<CommentOut<CreateComment>>,
) -> Result<HttpResponse, Error> {
  match db.article.get_by_slug(&auth, &slug).await? {
    Some(article) => {
      if cfg.allow_comments {
        match db.comment.store(&auth, article.id, &req.comment).await? {
          Some(comment_id) => {
            match db.comment.get_comment_by_id(&auth, comment_id).await? {
              Some(comment) => {
                Ok(HttpResponse::Ok().json(CommentOut {
                  comment,
                }))
              },
              None => {
                Ok(HttpResponse::InternalServerError().json(json!({
                  "error": "Failed to find new comment.",
                })))
              }
            }
          },
          None => {
            Ok(HttpResponse::InternalServerError().json(json!({
              "error": "Failed to add comment.",
            })))
          }
        }
      } else {
        Ok(HttpResponse::Forbidden().json(json!({
          "error": "Add comments disabled.",
        })))
      }
    },
    None => {
      Ok(HttpResponse::NotFound().json(json!({
        "error": "Comment not found",
      })))
    }
  }
}

/// delete an article comment
#[delete("/articles/{slug}/comments/{id}", wrap="Auth::required()")]
async fn delete_comment(
  auth: AuthData,
  cfg: web::Data<ArticleService>,
  db: web::Data<DbService>,
  info: web::Path<(String, i32)>,
) -> Result<HttpResponse, Error> {
  match db.comment.get_comment_by_id(&auth, info.1).await? {
    Some(comment) => {
      // Check if the user can delete the comment.
      if cfg.allow_comments && comment.author.user_id == auth.user_id {
        db.comment.delete(comment.id).await?;
        Ok(HttpResponse::Ok().finish())
      } else {
        Ok(HttpResponse::Forbidden().json(json!({
          "error": "Comments disabled.",
        })))
      }
    },
    None => {
      Ok(HttpResponse::NotFound().json(json!({
        "error": "Comment not found",
      })))
    }
  }
}

/////////////////////////////// Article Favorites

/// favorite article
#[post("/articles/{slug}/favorite", wrap="Auth::required()")]
async fn favorite(
  auth: AuthData,
  db: web::Data<DbService>,
  slug: web::Path<String>,
) -> Result<HttpResponse, Error> {
  match db.article.get_by_slug(&auth, &slug).await? {
    Some(mut article) => {
      // Check if the current user has already favorited the article
      if !article.favorited {
        // mark article as favorited by the current user
        db.article.favorite(&auth, article.id).await?;
        article.favorited = true;
        article.favorites_count += 1;
      }
      Ok(HttpResponse::Ok().json(ArticleOut::<ArticleDetails> {
        article,
      }))
    },
    None => {
      Ok(HttpResponse::NotFound().json(json!({
        "error": "Article not found",
      })))
    }
  }
}

/// unfavorite article
#[delete("/articles/{slug}/favorite", wrap="Auth::required()")]
async fn unfavorite(
  auth: AuthData,
  db: web::Data<DbService>,
  slug: web::Path<String>,
) -> Result<HttpResponse, Error> {
  match db.article.get_by_slug(&auth, &slug).await? {
    Some(mut article) => {
      // Check if the current user has already favorited the article
      if article.favorited {
        // mark article as unfavorited by the current user
        db.article.unfavorite(&auth, article.id).await?;
        article.favorited = false;
        article.favorites_count -= 1;
      }
      Ok(HttpResponse::Ok().json(ArticleOut::<ArticleDetails> {
        article,
      }))
    },
    None => {
      Ok(HttpResponse::NotFound().json(json!({
        "error": "Article not found",
      })))
    }
  }
}

#[derive(Debug, Clone, Default)]
pub struct ArticleService {
  pub allow_update: bool,
  pub allow_delete: bool,

  pub allow_comments: bool,
}

impl super::Service for ArticleService {
  fn load_app_config(&mut self, config: &AppConfig, _prefix: &str) -> Result<()> {
    self.allow_update = config.get_bool("Article.allow_update")?.unwrap_or(false);
    self.allow_delete = config.get_bool("Article.allow_delete")?.unwrap_or(false);

    self.allow_comments = config.get_bool("Article.allow_comments")?.unwrap_or(false);
    Ok(())
  }

  fn api_config(&self, web: &mut web::ServiceConfig) {
    web
      .data(self.clone())
      .service(list)
      .service(feed)

      // Article get/create/update/delete
      .service(get_article)
      .service(store_article)
      .service(update_article)
      .service(delete_article)

      // Article comments
      .service(get_comments)
      .service(store_comment)
      .service(delete_comment)

      // Article favorites
      .service(favorite)
      .service(unfavorite);
  }
}

pub fn new_factory() -> ArticleService {
  Default::default()
}
