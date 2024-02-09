CREATE TABLE IF NOT EXISTS packages (
	attribute TEXT NOT NULL,
	store_path TEXT NOT NULL,
	name TEXT,
	version TEXT,
	description TEXT,
	homepage TEXT,
	long_description TEXT,
	PRIMARY KEY (attribute)
)

