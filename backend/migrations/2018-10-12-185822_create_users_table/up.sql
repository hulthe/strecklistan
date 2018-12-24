CREATE TABLE users (
    name VARCHAR(64) PRIMARY KEY,
    display_name VARCHAR(128),
    salted_pass VARCHAR(256) NOT NULL,
    hash_iterations INTEGER NOT NULL CHECK (hash_iterations > 0)
)

