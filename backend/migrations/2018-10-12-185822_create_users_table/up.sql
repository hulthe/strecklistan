CREATE TABLE users (
    name VARCHAR(64) PRIMARY KEY,
    display_name VARCHAR(128),
    salted_pass VARCHAR(128) NOT NULL
)

