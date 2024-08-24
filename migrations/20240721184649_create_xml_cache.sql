-- Create BLOB
CREATE TABLE xml_cache(
    id uuid NOT NULL,
    name TEXT NOT NULL UNIQUE,
    content BLOB NOT NULL,
    PRIMARY KEY (id)
);
