default:
    @just --list

# Auto-format the source tree
fmt:
    treefmt

# Run cargo clippy
clippy:
    cargo clippy

# Run 'cargo run' on the project (e.g;: `j run -t ~/code/templates#\*home\*`)
run *ARGS:
    rm -rf ./tmp && cargo run -- ./tmp {{ARGS}} && set -x; lsd --tree ./tmp

# Run 'cargo watch' to run the project (auto-recompiles)
watch *ARGS:
    cargo watch -x "run -- {{ARGS}}"