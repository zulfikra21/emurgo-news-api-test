cargo clean;
cargo build --release;

mv target/release/actix-scaffolding ./actix-scaffolding;
cargo clean;

sudo docker compose build && sudo docker compose up -d;