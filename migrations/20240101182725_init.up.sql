-- Add up migration script here
CREATE TABLE IF NOT EXISTS events (
  id serial PRIMARY KEY,
  timestamp timestamptz NOT NULL,
  raw jsonb NOT NULL
);
