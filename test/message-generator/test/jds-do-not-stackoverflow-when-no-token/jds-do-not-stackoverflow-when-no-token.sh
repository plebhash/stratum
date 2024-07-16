cd roles
cargo build -p pool_sv2
cargo build -p jd_client
cargo build -p mining_proxy_sv2

cd ../utils/message-generator/
cargo build

RUST_LOG=debug cargo run ../../test/message-generator/test/jds-do-not-stackoverflow-when-no-token/jds-do-not-stackoverflow-when-no-token.json || { echo 'mg test failed' ; exit 1; }

sleep 10
