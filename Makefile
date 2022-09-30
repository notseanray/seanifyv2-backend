cleandb:
	rm seanify.db*

createdb:
	sqlx database create
	sqlx migrate run
