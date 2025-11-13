ESP_TARGET := "xtensa-esp32s3-none-elf"
ESP_CFG := "-Zbuild-std=\"core,alloc\""

# Generate the changelog
changelog path="CHANGELOG.md":
    git-cliff --output {{path}}

# Run the clippy linter
clippy:
    cargo clippy --workspace -- -D warnings

# Build the project
build mode="release":
    cargo build --package=kerfur --profile={{mode}} --target={{ESP_TARGET}} {{ESP_CFG}}

# Run the project
run mode="release":
    cargo run --package=kerfur --profile={{mode}} --target={{ESP_TARGET}} {{ESP_CFG}}

alias sim := simulate
# Run the project simulator
simulate mode="release":
      cargo run --package=kerfur-simulator --profile={{mode}}

# Check all project dependencies
deny:
    cargo deny check all

# Run all workspace tests
test:
    cargo test --workspace

# Check all files for typos
typos:
    typos

# Update all dependencies
update:
    cargo update --verbose
    @echo '{{CYAN+BOLD}}note{{NORMAL}}: or, if you have `just` installed, run `just inspect <dep>@<ver>`'

# Show the dependency tree for a specific package
inspect package="kerfur":
    cargo tree --invert --package={{package}}

# Update and run all checks
pre-commit: (update) (deny) (typos) (clippy) (test)
    @echo '{{GREEN+BOLD}}Success!{{NORMAL}} All checks passed!'
