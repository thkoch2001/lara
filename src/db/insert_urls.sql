WITH sel AS (
   SELECT val.domain_name, val.path, val.query, d.domain_id AS domain_id
   FROM  (
      VALUES {}
      ) val (domain_name, path, query)
   LEFT JOIN domain d ON (domain_name = d.name)
   )
, ins AS (
   INSERT INTO domain (name)
   SELECT DISTINCT domain_name FROM sel WHERE domain_id IS NULL
   RETURNING domain_id, name as domain_name
   )
INSERT INTO url (domain_id, path, query)
  SELECT
    COALESCE(sel.domain_id, ins.domain_id),
    sel.path,
    sel.query
  FROM sel
  LEFT JOIN ins USING (domain_name)
ON CONFLICT DO NOTHING
-- RETURNING url_id as id
;
