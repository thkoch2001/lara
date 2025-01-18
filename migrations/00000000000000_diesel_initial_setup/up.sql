-- This file was automatically created by Diesel to setup helper functions
-- and other internal bookkeeping. This file is safe to edit, any future
-- changes will be added to existing projects as new migrations.




-- Sets up a trigger for the given table to automatically set a column called
-- `updated_at` whenever the row is modified (unless `updated_at` was included
-- in the modified columns)
--
-- # Example
--
-- ```sql
-- CREATE TABLE users (id SERIAL PRIMARY KEY, updated_at TIMESTAMP NOT NULL DEFAULT NOW());
--
-- SELECT diesel_manage_updated_at('users');
-- ```
CREATE OR REPLACE FUNCTION diesel_manage_updated_at(_tbl regclass) RETURNS VOID AS $$
BEGIN
    EXECUTE format('CREATE TRIGGER set_updated_at BEFORE UPDATE ON %s
                    FOR EACH ROW EXECUTE PROCEDURE diesel_set_updated_at()', _tbl);
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION diesel_set_updated_at() RETURNS trigger AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD AND
        NEW.updated_at IS NOT DISTINCT FROM OLD.updated_at
    ) THEN
        NEW.updated_at := current_timestamp;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;


-- TODO
-- - check for alignment
--   - https://www.percona.com/blog/postgresql-column-alignment-and-padding-how-to-improve-performance-with-smarter-table-design/
-- https://webmasters.stackexchange.com/questions/16996/maximum-domain-name-length
CREATE TABLE domain (
  domain_id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  name TEXT NOT NULL
    CONSTRAINT is_lowercase CHECK (name = lower(name))
    CONSTRAINT is_trimmed CHECK (name = trim(name))
    CONSTRAINT is_unicode CHECK (unicode_assigned(name)) --requires pg 17
    CONSTRAINT maxlength CHECK (length(name) < 255)
    UNIQUE
);

-- https://stackoverflow.com/questions/417142/what-is-the-maximum-length-of-a-url-in-different-browsers
-- https://stackoverflow.com/questions/4229805/how-much-disk-space-is-needed-to-store-a-null-value-using-postgresql-db
-- TODO add rest TEXT, column encodes port number, http vs. https, authentication
-- TODO (long term) partition by domain? https://www.postgresql.org/docs/current/ddl-partitioning.html
CREATE TABLE url (
  url_id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  domain_id INTEGER NOT NULL REFERENCES domain,
  path TEXT DEFAULT '' NOT NULL,
  query TEXT,
  crawl_depth SMALLINT DEFAULT 0,
  crawl_priority REAL DEFAULT 1, -- 0: don't crawl
  http_etag TEXT, -- caching header
  http_last_modified TIMESTAMPTZ, -- caching header
  UNIQUE NULLS NOT DISTINCT (domain_id, path, query)
);


