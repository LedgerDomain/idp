# This silly rule is to make the error output of rustc parseable by KDevelop for easy error location clicking.
# Note that the `idp` directory is removed in the sed rule so that KDevelop will open the correct relative path.
.PHONY: rs-build
rs-build:
	cargo test --no-run 2>&1 | sed 's/ *--> \(.*\)/\1: error:/g'

.PHONY: rs-test
rs-test:
	RUST_BACKTRACE=1 cargo test 2>&1 | sed 's/ *--> \(.*\)/\1: error:/g'

.PHONY: rs-test-nocapture
rs-test-nocapture:
	RUST_BACKTRACE=1 cargo test -- --nocapture 2>&1 | sed 's/ *--> \(.*\)/\1: error:/g'
