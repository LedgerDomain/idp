# This silly rule is to make the error output of rustc parseable by KDevelop for easy error location clicking.
# Note that the `idp` directory is removed in the sed rule so that KDevelop will open the correct relative path.
.PHONY: rs-idp-proto-build
rs-idp-proto-build:
	cargo test --no-run 2>&1 | sed 's/ *--> idp_proto\/\(.*\)/\1: error:/g'

.PHONY: rs-idp-proto-test
rs-idp-proto-test:
	RUST_BACKTRACE=1 cargo test 2>&1 | sed 's/ *--> idp_proto\/\(.*\)/\1: error:/g'

.PHONY: rs-idp-proto-test-nocapture
rs-idp-proto-test-nocapture:
	RUST_BACKTRACE=1 cargo test -- --nocapture 2>&1 | sed 's/ *--> idp_proto\/\(.*\)/\1: error:/g'
