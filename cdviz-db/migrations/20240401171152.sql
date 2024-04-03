-- Create index "idx_subject" to table: "cdevents_lake"
CREATE INDEX "idx_subject" ON "cdevents_lake" ("subject");
-- Create index "idx_timestamp" to table: "cdevents_lake"
CREATE INDEX "idx_timestamp" ON "cdevents_lake" ("timestamp");
-- Set comment to column: "timestamp" on table: "cdevents_lake"
COMMENT ON COLUMN "cdevents_lake" ."timestamp" IS 'timestamp of cdevents extracted from context.timestamp in the json';
-- Set comment to column: "version" on table: "cdevents_lake"
COMMENT ON COLUMN "cdevents_lake" ."version" IS 'the version of the suject s type, extracted from context.type. The verion number are split in 0 for major, 1 for minor, 2 for patch';
