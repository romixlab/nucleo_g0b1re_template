# Nucleo G0B1RE template: the firmware (firmware/, its .cargo/config.toml sets the target), the host workspace at
# the root (API crate, client library, `g0b1re` command line, Python module).

chip := "STM32G0B1RETx"
fw_dir := "firmware/target/thumbv6m-none-eabi"

default:
    @just --list

# Host tests (the workspace; the Python module's tests: `just test-py`)
test *flags:
    cargo test --workspace --locked {{flags}}

# Format check and clippy for the host workspace and the firmware; the firmware's release build must fit
lint:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets --locked -- -D warnings
    cd firmware && cargo fmt --all --check
    cd firmware && cargo clippy --bins --locked -- -D warnings
    cd firmware && cargo build --release --locked

# Format the host workspace and the firmware
fmt:
    cargo fmt --all
    cd firmware && cargo fmt --all

# Build, flash and run the dev firmware with its defmt log (probe-rs); Ctrl-C stops the log, the firmware keeps running
run *args:
    cd firmware && cargo run {{args}}

# Build the release firmware
build-fw:
    cd firmware && cargo build --release --locked

# Flash the release firmware through the probe and reset the board
flash: build-fw
    probe-rs download --chip {{chip}} {{fw_dir}}/release/nucleo_g0b1re
    probe-rs reset --chip {{chip}}

# Attach to the running dev firmware's defmt log without flashing (stop it before using `g0b1re`: one probe, one user)
attach:
    probe-rs attach --chip {{chip}} {{fw_dir}}/debug/nucleo_g0b1re

# Build the `g0b1re` command line and install it on this PC (with tpm on PATH through tpm install, else cargo install)
install:
    cargo build --profile deploy --locked -p nucleo_g0b1re_cli
    if command -v tpm >/dev/null; then tpm install target/deploy/g0b1re; else cargo install --locked --path nucleo_g0b1re_cli; fi

# Bring this PC and the board on the probe up to main: pull --ff-only (refuses on local changes), install, flash, check `g0b1re --version`
deploy:
    #!/usr/bin/env bash
    set -eu
    dirty=$(git status --porcelain)
    if [ -n "$dirty" ]; then echo "local changes, not deploying:"; echo "$dirty"; exit 1; fi
    git pull --ff-only
    just install
    just flash
    g0b1re --version | grep -q "($(git rev-parse --short HEAD)" || { echo "g0b1re --version does not show $(git rev-parse --short HEAD):"; g0b1re --version; exit 1; }
    g0b1re --version

# Build the Python module into nucleo_g0b1re_py/.venv (when Rust sources changed) and open a prompt with the board as `dev`
py *args:
    cd nucleo_g0b1re_py && uv sync && uv run python -i -c "import nucleo_g0b1re; dev = nucleo_g0b1re.Device({{args}}); print(dev, dev.firmware())"

# Run the Python example: blink the LED and print what runs on the board
py-example *args:
    cd nucleo_g0b1re_py && uv sync && uv run python examples/blink.py {{args}}

# Build the Python module and run its tests (no board needed)
test-py:
    cd nucleo_g0b1re_py && uv run --group dev pytest

# Give the project your own name: `just rename my_board` (crates, directories, the command line name, env vars)
rename name short="":
    uv run scripts/rename.py {{name}} {{short}}
