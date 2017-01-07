CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    display_name TEXT NOT NULL,
    email TEXT UNIQUE NOT NULL,
    code TEXT NOT NULL
);

COMMENT ON COLUMN users.code IS 'Base32 key for OTP code generation';