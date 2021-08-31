# This silly rule is to make the error output of rustc parseable by KDevelop for easy error location clicking.
# Note that the `idp` directory is removed in the sed rule so that KDevelop will open the correct relative path.
.PHONY: rs-idp-core-build
rs-idp-core-build:
	cargo test --no-run 2>&1 | sed 's/ *--> idp_core\/\(.*\)/\1: error:/g'

.PHONY: rs-idp-core-build-notests
rs-idp-core-build-notests:
	cargo build 2>&1 | sed 's/ *--> idp_core\/\(.*\)/\1: error:/g'

.PHONY: rs-idp-core-test
rs-idp-core-test:
	RUST_BACKTRACE=1 cargo test 2>&1 | sed 's/ *--> idp_core\/\(.*\)/\1: error:/g'

.PHONY: rs-idp-core-test-nocapture
rs-idp-core-test-nocapture:
	RUST_BACKTRACE=1 cargo test -- --nocapture 2>&1 | sed 's/ *--> idp_core\/\(.*\)/\1: error:/g'
