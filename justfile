local_build:
    cargo build --release && cp target/release/hop ~/.local/bin/

check:
    cargo clippy

fix:
    cargo fmt --all -- --check
    cargo fix --allow-dirty
