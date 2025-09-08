all: run

run:
	@docker compose down
	@docker compose build
	@docker compose up -d

build:
	@docker compose down
	@docker compose build

logs:
	@docker compose logs -f dnsliar valkey

start:
	@docker compose start

stop:
	@docker compose stop

godnsliar:
	@docker exec -it dnsliar sh

govalkey:
	@docker exec -it valkey sh

.PHONY: run build stop start logs build godnsliar govalkey

