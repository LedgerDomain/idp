# This silly rule is to make the error output of rustc parseable by KDevelop for easy error location clicking.
# Note that the `idp` directory is removed in the sed rule so that KDevelop will open the correct relative path.
.PHONY: rs-idp-client-build
rs-idp-client-build:
	cargo test --no-run 2>&1 | sed 's/ *--> idp_client\/\(.*\)/\1: error:/g'

.PHONY: rs-idp-client-test
rs-idp-client-test:
	RUST_BACKTRACE=1 cargo test 2>&1 | sed 's/ *--> idp_client\/\(.*\)/\1: error:/g'

.PHONY: rs-idp-client-test-nocapture
rs-idp-client-test-nocapture:
	RUST_BACKTRACE=1 cargo test -- --nocapture 2>&1 | sed 's/ *--> idp_client\/\(.*\)/\1: error:/g'
