-- Enable citext extension so we don't have to worry about lowercasing emails
CREATE EXTENSION IF NOT EXISTS citext;

CREATE TABLE subscriptions(
    id uuid NOT NULL PRIMARY KEY,
    email CITEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    subscribed_at timestamptz NOT NULL
);
