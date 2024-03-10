-- Add up migration script here
CREATE TABLE IF NOT EXISTS `cdevents_lake` (
  `id` BIGINT GENERATED ALWAYS AS IDENTITY,
  `created_at` TIMESTAMPZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `timestamp` TIMESTAMPZ NOT NULL,
  `payload` JSONB NOT NULL,
  `subject` VARCHAR(100) NOT NULL,
  `predicate` VARCHAR(100) NOT NULL,
);


-- create a view based on fields in the json payload
-- source: [Postgresql json column to view - Database Administrators Stack Exchange](https://dba.stackexchange.com/questions/151838/postgresql-json-column-to-view?newreg=ed0a9389843a45699bfb02559dd32038)
-- DO $$
-- DECLARE l_keys text;
-- BEGIN
--   drop view if exists YOUR_VIEW_NAME cascade;

--   select string_agg(distinct format('jerrayel ->> %L as %I',jkey, jkey), ', ')
--       into l_keys
--   from cdevents_lake, jsonb_array_elements(payload) as t(jerrayel), jsonb_object_keys(t.jerrayel) as a(jkey);

--   execute 'create view cdevents_flatten as select '||l_keys||' from cdevents_lake, jsonb_array_elements(payload) as t(jerrayel)';
-- END$$;
