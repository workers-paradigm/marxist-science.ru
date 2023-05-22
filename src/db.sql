BEGIN;

CREATE TABLE IF NOT EXISTS versions (
       version SERIAL8 NOT NULL UNIQUE
);

CREATE OR REPLACE PROCEDURE new_version(current INTEGER, commands TEXT)
LANGUAGE plpgsql
AS $$
BEGIN
  IF NOT EXISTS (SELECT version FROM versions WHERE version = current)
  THEN
    EXECUTE commands;
    INSERT INTO versions (version) VALUES (current);
  END IF;
END; $$;

CALL new_version(1, $version$
  CREATE TABLE IF NOT EXISTS users (
         id serial PRIMARY KEY,
         name varchar NOT NULL,
         pass_hash varchar NOT NULL
  );

  CREATE TABLE IF NOT EXISTS sessions (
         id serial PRIMARY KEY,
         user_id serial8 REFERENCES users(id) ON DELETE CASCADE,
         content varchar NOT NULL,
         expires date NOT NULL
  );

  -- USERS AND SESSIONS [END]

  CREATE TABLE IF NOT EXISTS files (
         id serial PRIMARY KEY,
         url varchar NOT NULL,
         md5 varchar NOT NULL
  );

  CREATE TABLE IF NOT EXISTS articles (
         id serial PRIMARY KEY,
         title varchar,
         markdown integer REFERENCES files(id) ON DELETE CASCADE,
         cover integer REFERENCES files(id) ON DELETE CASCADE
  );

  CREATE TABLE IF NOT EXISTS books (
         id serial PRIMARY KEY,
         title varchar NOT NULL,
         author varchar,
         pub_date date
  );

  CREATE TABLE IF NOT EXISTS books_files (
    book integer REFERENCES books(id) ON DELETE CASCADE,
    file integer REFERENCES files(id) ON DELETE CASCADE,
    CONSTRAINT books_files_pkey PRIMARY KEY (book, file)
  );

  -- BOOKS AND ARTICLES [END]

  CREATE TABLE IF NOT EXISTS sectors (
         id serial PRIMARY KEY,
         cover integer REFERENCES files(id) ON DELETE CASCADE,
         name varchar
  );

  CREATE TABLE IF NOT EXISTS articles_sectors (
         article integer REFERENCES articles(id) ON DELETE CASCADE,
         sector integer REFERENCES sectors(id) ON DELETE CASCADE,
         CONSTRAINT articles_sectors_pkey PRIMARY KEY (article, sector)
  );

  CREATE TABLE IF NOT EXISTS books_sectors (
         book integer REFERENCES books(id) ON DELETE CASCADE,
         sector integer REFERENCES sectors(id) ON DELETE CASCADE,
         CONSTRAINT books_sectors_pkey PRIMARY KEY (book, sector)
  );

  CREATE TABLE IF NOT EXISTS tags (
         id serial PRIMARY KEY,
         name varchar
  );

  CREATE TABLE IF NOT EXISTS articles_tags (
         article integer REFERENCES articles(id) ON DELETE CASCADE,
         tag integer REFERENCES tags(id) ON DELETE CASCADE,
         CONSTRAINT articles_tags_pkey PRIMARY KEY (article, tag)
  );

  CREATE TABLE IF NOT EXISTS books_tags (
         book integer REFERENCES books(id) ON DELETE CASCADE,
         tag integer REFERENCES tags(id) ON DELETE CASCADE,
         CONSTRAINT books_tags_pkey PRIMARY KEY (book, tag)
  );

  -- BOOKS+ARTICLES AND SECTORS+TAGS [END]

$version$); -- version 1

CALL new_version(2, $version$
  CREATE VIEW valid_sessions AS
    SELECT id, user_id, content
    FROM sessions
    WHERE expires > (SELECT CURRENT_DATE);

  CREATE VIEW next_month AS
    SELECT CURRENT_DATE + 30 AS next_month;

  CREATE OR REPLACE PROCEDURE delay_session_expiration (id integer)
  LANGUAGE SQL
  AS $$
    UPDATE sessions
    SET expires = (SELECT * FROM next_month)
    WHERE sessions.id = id;
  $$;

  CREATE OR REPLACE FUNCTION new_session (user_id integer, content varchar)
  RETURNS TABLE (session varchar, user_id integer, expires date)
  LANGUAGE SQL
  AS $$
    INSERT INTO sessions (user_id, content, expires)
    VALUES (user_id, content, (SELECT * FROM next_month))
    RETURNING content, user_id, expires;
  $$;

  -- USERS AND SESSIONS [END]

  CREATE VIEW sectors_names_covers AS
    SELECT sectors.id, name, url
    FROM sectors
    LEFT JOIN files ON files.id = sectors.cover;

  CREATE OR REPLACE FUNCTION get_articles_in_sector (requested_sector integer)
  RETURNS TABLE (id integer, title varchar, cover_url varchar)
  LANGUAGE SQL
  AS $$
    SELECT articles.id, title, url
    FROM articles, articles_sectors, files
    WHERE articles_sectors.sector = requested_sector
      AND articles.id = articles_sectors.article
      AND files.id = articles.cover;
  $$;

  CREATE OR REPLACE FUNCTION get_article_tags (requested_article integer)
  RETURNS TABLE (id integer, name varchar)
  LANGUAGE SQL
  AS $$
    SELECT tags.id, tags.name
    FROM articles_tags, tags
    WHERE articles_tags.article = requested_article
      AND tags.id = articles_tags.tag;
  $$;

  CREATE OR REPLACE FUNCTION get_article_info (article integer)
  RETURNS TABLE (title varchar, markdown_url varchar, cover_url varchar)
  LANGUAGE SQL
  AS $$
    SELECT title, md_files.url, cover_files.url
    FROM articles, files AS md_files, files AS cover_files
    WHERE articles.id = article
      AND md_files.id = articles.markdown
      AND cover_files.id = articles.cover;
  $$;

  -- ARTICLES + SECTORS + TAGS [END]

  CREATE OR REPLACE FUNCTION get_books_in_sector (requested_sector integer)
  RETURNS TABLE (id integer, title varchar)
  LANGUAGE SQL
  AS $$
    SELECT id, title
    FROM books, books_sectors
    WHERE books_sectors.sector = requested_sector
      AND books.id = books_sectors.book;
  $$;

  CREATE OR REPLACE FUNCTION get_book_tags (requested_book integer)
  RETURNS TABLE (id integer, name varchar)
  LANGUAGE SQL
  AS $$
    SELECT tags.id, tags.name
    FROM books_tags, tags
    WHERE books_tags.book = requested_book
      AND tags.id = books_tags.tag;
  $$;

  CREATE OR REPLACE FUNCTION get_book_file_urls (requested_book integer)
  RETURNS TABLE (url varchar)
  LANGUAGE SQL
  AS $$
    SELECT files.url
    FROM books_files, files
    WHERE books_files.book = requested_book
      AND files.id = books_files.file;
  $$;

  -- BOOKS + SECTORS + TAGS [END]


  CREATE OR REPLACE FUNCTION insert_article (title varchar, md integer, cover integer)
  RETURNS integer
  LANGUAGE SQL
  AS $$
    INSERT INTO articles (title, markdown, cover)
    VALUES (title, md, cover)
    RETURNING articles.id;
  $$;

  CREATE OR REPLACE FUNCTION insert_book (title varchar, author varchar, pub_date date)
  RETURNS integer
  LANGUAGE SQL
  AS $$
    INSERT INTO books (title, author, pub_date)
    VALUES (title, author, pub_date)
    RETURNING books.id;
  $$;

$version$); -- version 2


COMMIT;
