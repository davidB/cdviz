-- Drop index "idx_timestamp" from table: "cdevents_lake"
DROP INDEX "idx_timestamp";
-- Create index "idx_timestamp" to table: "cdevents_lake"
CREATE INDEX "idx_timestamp" ON "cdevents_lake" USING brin ("timestamp");
-- Create index "idx_cdevents" to table: "cdevents_lake"
CREATE INDEX "idx_cdevents" ON "cdevents_lake" USING gin ("payload");
