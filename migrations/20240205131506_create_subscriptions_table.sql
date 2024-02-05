-- Add migration script here
CREATE TABLE subscriptions(
    id UUID NOT NULL PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    subscribed_at TIMESTAMPTZ NOT NULL -- time stamp with time zone
);