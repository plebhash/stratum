cd roles
cargo build -p pool_sv2
cargo build -p jd_server
cargo build -p jd_client
cargo build -p translator_sv2
cargo build -p sv1-mining-device

cd ../utils/message-generator/
cargo build

RUST_LOG=debug cargo run ../../test/message-generator/test/interop-jd-translator/interop-jd-translator.json || { echo 'mg test failed' ; exit 1; }

sleep 10
