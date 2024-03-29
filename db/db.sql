-- \pset format wrapped
-- \pset columns 80
-- \pset linestyle unicode

BEGIN;

CREATE TABLE IF NOT EXISTS versions (
       version serial PRIMARY KEY,
       down text NOT NULL
);

CREATE OR REPLACE PROCEDURE new_version(current INTEGER, up TEXT, down TEXT)
LANGUAGE plpgsql
AS $$
BEGIN
  IF NOT EXISTS (SELECT version FROM versions WHERE version = current)
  THEN
    EXECUTE up;
    INSERT INTO versions (version, down) VALUES (current, down);
  END IF;
END; $$;

--- VERSION 1: Create an articles table. Undoing deletes the articles table
CALL new_version(1, $up$
  CREATE TABLE articles (
         id serial PRIMARY KEY,
         title text NOT NULL DEFAULT 'Черновик',
         cover text,
         contents text NOT NULL DEFAULT '{ "blocks" : [ { "type": "paragraph" } ] }',
         published boolean NOT NULL DEFAULT false,
         created_at timestamp NOT NULL DEFAULT NOW()
  );
$up$,
$down$
  DROP TABLE articles;
$down$); -- version 1:

-- VERSION 2: Create files table.
CALL new_version(2, $up$
  CREATE TABLE files (
         id bytea PRIMARY KEY NOT NULL,
         title text NOT NULL,
         ext text NOT NULL
  );
$up$,
$down$
  DROP TABLE files;
$down$); -- version 2

-- VERSION 3: Archive
CALL new_version(3, $up$
  CREATE TABLE archive_entries (
         id serial PRIMARY KEY,
         title text NOT NULL DEFAULT 'UNTITLED',
         author text NOT NULL DEFAULT 'Наука Марксизм',
         cover text,
         description text NOT NULL DEFAULT '',
         created_at timestamp NOT NULL DEFAULT NOW()
  );

  CREATE TABLE archive_entries_files (
         entry integer REFERENCES archive_entries (id) ON DELETE CASCADE,
         file bytea REFERENCES files (id) ON DELETE CASCADE,
         PRIMARY KEY (entry, file)
  );
$up$,
$down$
  DROP TABLE archive_entries;
$down$);

CALL new_version(4, $up$
  CREATE TABLE users (
         id serial PRIMARY KEY,
         username text NOT NULL,
         password_hash text NOT NULL
  );
$up$,
$down$
  DROP TABLE users;
$down$);

CALL new_version(5, $up$
  ALTER TABLE articles ADD COLUMN author text NOT NULL DEFAULT '';
$up$,
$down$
  ALTER TABLE articles DROP COLUMN author;
$down$);

CALL new_version(6, $up$
  CREATE TABLE rubrics(
         id serial PRIMARY KEY,
         title text NOT NULL DEFAULT 'NEED RUBRIC NAME',
         cover text
  );

  CREATE TABLE articles_rubrics(
         article integer NOT NULL REFERENCES articles(id) ON DELETE CASCADE,
         rubric integer NOT NULL REFERENCES rubrics(id) ON DELETE CASCADE,
         PRIMARY KEY (article, rubric)
  );
$up$,
$down$
  DROP TABLE rubrics;
  DROP TABLE articles_rubrics;
$down$);

CALL new_version(7, $up$
  CREATE TABLE sessions(
    id bytea PRIMARY KEY,
    user_id integer REFERENCES users(id) ON DELETE CASCADE,
    expires_at timestamp NOT NULL DEFAULT NOW() + interval '1 week'
  );
$up$,
$down$
  DROP TABLE sessions;
$down$);

COMMIT;
