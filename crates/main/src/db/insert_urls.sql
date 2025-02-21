WITH
  outlinks (domain_name, path, query, context) AS (
    VALUES
    {}
  ),
  domain_select AS (
    SELECT o.domain_name, o.path, o.query, d.domain_id AS domain_id
    FROM outlinks o
    LEFT JOIN domain d ON (domain_name = d.name)
  ),
  domain_insert AS (
    INSERT INTO domain (name)
    SELECT DISTINCT domain_name FROM domain_select WHERE domain_id IS NULL
    RETURNING domain_id, name as domain_name
  ),
  domain_ids AS (
    SELECT
      COALESCE(domain_select.domain_id, domain_insert.domain_id) AS domain_id,
      domain_select.path,
      domain_select.query,
      domain_name
    FROM domain_select
    LEFT JOIN domain_insert USING (domain_name)
  ),
  url_select AS (
   SELECT url_id, domain_id, path, query
   FROM url
   JOIN domain_ids USING (domain_id, path, query)
  ),
  url_insert AS (
    INSERT INTO url (domain_id, path, query)
    SELECT domain_id, path, query
    FROM domain_ids
    ON CONFLICT DO NOTHING
    RETURNING url_id, domain_id, path, query
  ),
  fr_insert AS (
    INSERT INTO fetch_result (url_id, time, http_status)
    VALUES ({})
    RETURNING fetch_result_id
  )
INSERT INTO inlink (fetch_result_id, url_id, context)
  SELECT
    fr_insert.fetch_result_id,
    url_id,
    context
  FROM
    fr_insert,
    (SELECT * FROM url_insert UNION SELECT * FROM url_select)
    JOIN domain_ids USING (domain_id, path, query)
    JOIN outlinks USING (domain_name, path, query)
;
