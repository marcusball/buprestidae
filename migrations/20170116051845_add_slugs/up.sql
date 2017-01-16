
/* Add the slug column, populating existing posts with a random string for their `slug`. */
ALTER TABLE posts 
ADD COLUMN slug VARCHAR(64) NOT NULL UNIQUE DEFAULT md5(''||now()::text||random()::text);

/* Remove the DEFAULT value */
ALTER TABLE posts ALTER COLUMN slug DROP DEFAULT;

COMMENT ON COLUMN posts.slug IS 'Unique, URL-safe string to be used for the post URL.';