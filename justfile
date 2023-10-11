alias f := fmt
alias t := test
alias b := build
alias r := run
alias rr := rrun
alias w := watch

default:
  just -l

fmt:
  rustup run nightly cargo fmt

build *cargo_args:
  cargo build {{cargo_args}}

run *args:
  cargo run -- {{args}}
rrun *args:
  cargo run --release -- {{args}}

watch *args:
  cargo watch -x run {{args}}

test:
  cargo test

install:
  cargo install --all-features --path .
