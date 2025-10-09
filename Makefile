all: run

run:
	@docker compose down -v
	@docker compose build
	@docker compose up -d
	@docker compose logs -f

down:
	@docker compose down -v

build:
	@docker compose down -v
	@docker compose build

logs:
	@docker compose logs -f

start:
	@docker compose start

stop:
	@docker compose stop

godnsliar:
	@docker exec -it dnsliar sh

govalkey:
	@docker exec -it valkey sh

.PHONY: run down build stop start logs build godnsliar govalkey

