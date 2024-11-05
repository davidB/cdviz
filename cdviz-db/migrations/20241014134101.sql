-- Create "cdevents_lake" table
CREATE TABLE "cdevents_lake" ("id" bigint NOT NULL GENERATED ALWAYS AS IDENTITY, "imported_at" timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP, "timestamp" timestamptz NOT NULL, "payload" jsonb NOT NULL, "subject" character varying(100) NOT NULL, "predicate" character varying(100) NOT NULL, "version" integer[] NULL, "context_id" character varying(100) NOT NULL, PRIMARY KEY ("id"));
-- Create index "cdevents_lake_context_id_key" to table: "cdevents_lake"
CREATE UNIQUE INDEX "cdevents_lake_context_id_key" ON "cdevents_lake" ("context_id");
-- Create index "idx_subject" to table: "cdevents_lake"
CREATE INDEX "idx_subject" ON "cdevents_lake" ("subject");
-- Create index "idx_timestamp" to table: "cdevents_lake"
CREATE INDEX "idx_timestamp" ON "cdevents_lake" ("timestamp");
-- Set comment to table: "cdevents_lake"
COMMENT ON TABLE "cdevents_lake" IS 'table of stored cdevents without transformation';
-- Set comment to column: "imported_at" on table: "cdevents_lake"
COMMENT ON COLUMN "cdevents_lake" ."imported_at" IS 'the timestamp when the cdevent was stored into the table';
-- Set comment to column: "timestamp" on table: "cdevents_lake"
COMMENT ON COLUMN "cdevents_lake" ."timestamp" IS 'timestamp of cdevents extracted from context.timestamp in the json';
-- Set comment to column: "payload" on table: "cdevents_lake"
COMMENT ON COLUMN "cdevents_lake" ."payload" IS 'the full cdevent in json format';
-- Set comment to column: "subject" on table: "cdevents_lake"
COMMENT ON COLUMN "cdevents_lake" ."subject" IS 'subject extracted from context.type in the json';
-- Set comment to column: "predicate" on table: "cdevents_lake"
COMMENT ON COLUMN "cdevents_lake" ."predicate" IS 'predicate of the subject, extracted from context.type in the json';
-- Set comment to column: "version" on table: "cdevents_lake"
COMMENT ON COLUMN "cdevents_lake" ."version" IS 'the version of the suject s type, extracted from context.type. The version number are split in 0 for major, 1 for minor, 2 for patch';
-- Set comment to column: "context_id" on table: "cdevents_lake"
COMMENT ON COLUMN "cdevents_lake" ."context_id" IS 'the id of the event, extracted from context.id';

create or replace procedure store_cdevent(
    cdevent jsonb
)
as $$
declare
    ts timestamp with time zone;
    tpe varchar(255);
    context_id varchar(100);
    tpe_subject varchar(100);
    tpe_predicate varchar(100);
    tpe_version INTEGER[3];
begin
    context_id := (cdevent -> 'context' ->> 'id');
    tpe := (cdevent -> 'context' ->> 'type');
    tpe_subject := SPLIT_PART(tpe, '.', 3);
    tpe_predicate := SPLIT_PART(tpe, '.', 4);
    tpe_version[0]:= SPLIT_PART(tpe, '.', 5)::INTEGER;
    tpe_version[1]:= SPLIT_PART(tpe, '.', 6)::INTEGER;
    tpe_version[2]:= SPLIT_PART(SPLIT_PART(tpe, '.', 7), '-', 1)::INTEGER;
    -- if (jsonb_typeof(cdevent -> 'context' ->> 'timestamp') = 'timestampz') then
        ts := (cdevent -> 'context' ->> 'timestamp')::timestamp with time zone;
    -- else
    --    raise exception 'Input Jsonb doesn not contain a valid timestamp';
    -- end if;
    insert into "cdevents_lake"("payload", "timestamp", "subject", "predicate", "version", "context_id") values(cdevent, ts, tpe_subject, tpe_predicate, tpe_version, context_id);
end;
$$ language plpgsql;
