alias t := test
alias ut:= unit-test
alias it := integration-test
unit-test: unit-test-jni-1_8 unit-test-jni-1_6

unit-test-jni-1_6:
  cargo test --features jni_1_6

unit-test-jni-1_8:
  cargo test --features jni_1_8

integration-test: integration-test-jni-1_8 integration-test-jni-1_6

integration-test-jni-1_6:
    (cd test-crates && cargo test --features jni_1_6)

integration-test-jni-1_8:
    (cd test-crates && cargo test --features jni_1_8)

test: unit-test integration-test

test-jni-1_8: unit-test-jni-1_8 integration-test-jni-1_8

test-jni-1_6: unit-test-jni-1_6 integration-test-jni-1_6

coverage-tools:
  rustup component add llvm-tools 
  cargo install cargo-binutils
  cargo install rustfilt

test_coverage := "test-coverage"
coverage_file := "duchess-coverage-%p-%10m.profraw"

coverage-clean:
  #!/usr/bin/env bash
  set -euxo pipefail
  target={{justfile_directory()}}/target
  coverage_dir=$target/{{test_coverage}}
  rm -rf $coverage_dir
  rm -rf $target/ui-coverage-report


coverage-unit-tests:
  #!/usr/bin/env bash
  set -euxo pipefail
  target={{justfile_directory()}}/target
  coverage_dir=$target/{{test_coverage}}
  (RUSTFLAGS="-C instrument-coverage" LLVM_PROFILE_FILE=$coverage_dir/{{coverage_file}} cargo test) || true


coverage-ui-test:
  #!/usr/bin/env bash
  set -euxo pipefail
  target={{justfile_directory()}}/target
  coverage_dir=$target/{{test_coverage}}
  (cd test-crates && RUSTFLAGS="-C instrument-coverage" LLVM_PROFILE_FILE=$coverage_dir/duchess-%p-%10m.profraw cargo test) || true

coverage-show:
  #!/usr/bin/env bash
  set -euxo pipefail
  target={{justfile_directory()}}/target
  coverage_dir=$target/{{test_coverage}}
  # For some reason, the LLVM tools are also emitting profiling data, suppress this.
  export LLVM_PROFILE_FILE=/dev/null
  echo "Found profile data from $(ls $coverage_dir/duchess*.profraw | wc -l) profile runs"
  rust-profdata merge -sparse $coverage_dir/duchess*.profraw -o $coverage_dir/test-crates.profdata
  # Determine the operating system
  OS="$(uname)"
  if [[ "$OS" == "Darwin" ]]; then
      FILE_EXTENSION="dylib"
  elif [[ "$OS" == "Linux" ]]; then
      FILE_EXTENSION="so"
  else
      echo "Unsupported OS"
      exit 1
  fi
  rust-cov show --instr-profile $coverage_dir/test-crates.profdata -Xdemangler=rustfilt \
    --format=html \
    --output-dir $target/ui-coverage-report \
    --object test-crates/target/tests/rust-to-java/examples/greeting \
    --object test-crates/target/tests/rust-to-java/exceptions \
       $( \
      for file in \
        $( \
          RUSTFLAGS="-C instrument-coverage" \
          LLVM_PROFILE_FILE="$target/ignored-%p-%10m.profraw" \
            cargo test --tests --no-run --message-format=json \
              | jq -r "select(.profile.test == true) | .filenames[]" \
              | grep -v dSYM - \
        ); \
      do \
        printf "%s %s " -object $file; \
      done \
    ) \
    --object $target/debug/deps/libduchess_macro-*.$FILE_EXTENSION \
    --show-instantiations=false \
    --sources {{justfile_directory()}}/src \
    --sources {{justfile_directory()}}/macro
  # Allow CI to suppress autoopening the report
  if [ -z "${NO_OPEN+x}" ]; then
    open $target/ui-coverage-report/index.html
  fi


coverage: coverage-clean coverage-unit-tests coverage-ui-test coverage-show

publish:
  git describe --exact-match --tags HEAD || (echo "You must publish a tagged release. Checkout the tag before publishing" && exit 1)
  (cd duchess-reflect && cargo publish)
  cd macro && cargo publish
  cargo publish
