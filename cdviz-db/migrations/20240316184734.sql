-- Create "cdevents_lake" table
CREATE TABLE "cdevents_lake" ("id" bigint NOT NULL GENERATED ALWAYS AS IDENTITY, "imported_at" timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP, "timestamp" timestamptz NOT NULL, "payload" jsonb NOT NULL, "subject" character varying(100) NOT NULL, "predicate" character varying(100) NOT NULL, "version" integer[] NULL, PRIMARY KEY ("id"));
-- Set comment to table: "cdevents_lake"
COMMENT ON TABLE "cdevents_lake" IS 'table of stored cdevents without transformation';
-- Set comment to column: "imported_at" on table: "cdevents_lake"
COMMENT ON COLUMN "cdevents_lake" ."imported_at" IS 'the timestamp when the cdevent was stored into the table';
-- Set comment to column: "payload" on table: "cdevents_lake"
COMMENT ON COLUMN "cdevents_lake" ."payload" IS 'the full cdevent in json format';
-- Set comment to column: "subject" on table: "cdevents_lake"
COMMENT ON COLUMN "cdevents_lake" ."subject" IS 'subject extracted from context.type in the json';
-- Set comment to column: "predicate" on table: "cdevents_lake"
COMMENT ON COLUMN "cdevents_lake" ."predicate" IS 'predicate of the subject, extracted from context.type in the json';
-- Set comment to column: "version" on table: "cdevents_lake"
COMMENT ON COLUMN "cdevents_lake" ."version" IS 'the version of the suject s type, extracted from context.type; the verion number are split in 0 for major, 1 for minor, 2 for patch';
