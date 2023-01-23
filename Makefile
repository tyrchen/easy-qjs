build:
	@BUILD_ICONS=1 cargo build

test:
	@cargo nextest run

release:
	@cargo release tag --execute
	@git cliff -o CHANGELOG.md
	@git commit -a -m "Update CHANGELOG.md" || true
	@git push origin master
	@cargo release push --execute

.PHONY: build test release
