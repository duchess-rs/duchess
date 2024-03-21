alias t := test
alias ut:= unit-test
alias it := integration-test
unit-test:
  cargo test

integration-test:
    (cd test-crates && cargo test)

test: unit-test integration-test