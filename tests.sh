cargo test -q -p o2o
cargo test -q -p o2o --no-default-features --features syn1
cargo test -q -p o2o --no-default-features --features syn2
cargo test -q -p o2o-tests --no-default-features --features syn1
cargo test -q -p o2o-tests --no-default-features --features syn2
cargo test -q -p o2o-impl --no-default-features --features syn
cargo test -q -p o2o-impl --no-default-features --features syn2