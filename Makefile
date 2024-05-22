build:
	bun install
	bunx tailwindcss -i ./templates/static/style.css -o ./templates/static/dist.css
	cargo build --release