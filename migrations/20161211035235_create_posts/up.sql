CREATE TABLE posts (
    id SERIAL PRIMARY KEY,
    title VARCHAR NOT NULL,
    body TEXT NOT NULL,
    is_published BOOLEAN NOT NULL DEFAULT 'f',
    publish_date TIMESTAMPTZ NULL,
    last_modification_date TIMESTAMPTZ NOT NULL
    CONSTRAINT valid_publish_date CHECK (NOT is_published  OR publish_date IS NOT NULL)
);