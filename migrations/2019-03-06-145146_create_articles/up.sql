CREATE TABLE articles (
    id SERIAL PRIMARY KEY,
    author_id INTEGER NOT NULL REFERENCES users (id),
    slug TEXT UNIQUE NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    body TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE INDEX articles_author_id_idx ON articles (author_id);
-- indices are already created for slugs, as slugs are unique as per the spec

SELECT diesel_manage_updated_at('articles');

CREATE TABLE favorite_articles (
    user_id INTEGER NOT NULL REFERENCES users (id),
    article_id INTEGER NOT NULL REFERENCES articles (id),
    PRIMARY KEY (user_id, article_id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE INDEX favorite_articles_user_id_idx ON favorite_articles (user_id);
CREATE INDEX favorite_articles_article_id_idx ON favorite_articles (article_id);

SELECT diesel_manage_updated_at('favorite_articles');
