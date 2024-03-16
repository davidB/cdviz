-- Add up migration script here
CREATE TABLE IF NOT EXISTS "cdevents_lake" (
  "id" BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  "imported_at" TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
  "timestamp" TIMESTAMP WITH TIME ZONE NOT NULL,
  "payload" JSONB NOT NULL,
  "subject" VARCHAR(100) NOT NULL,
  "predicate" VARCHAR(100) NOT NULL,
  "version" INTEGER[3]
);

COMMENT ON TABLE "cdevents_lake" IS 'table of stored cdevents without transformation';
COMMENT ON COLUMN "cdevents_lake"."imported_at" IS 'the timestamp when the cdevent was stored into the table';
COMMENT ON COLUMN "cdevents_lake"."payload" IS 'the full cdevent in json format';
COMMENT ON COLUMN "cdevents_lake"."subject" IS 'subject extracted from context.type in the json';
COMMENT ON COLUMN "cdevents_lake"."predicate" IS 'predicate of the subject, extracted from context.type in the json';
COMMENT ON COLUMN "cdevents_lake"."version" IS 'the version of the suject s type, extracted from context.type; the verion number are split in 0 for major, 1 for minor, 2 for patch';

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
