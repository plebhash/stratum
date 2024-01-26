#!/bin/sh

ROOT_DIR=$PWD
ROLES_DIR=$ROOT_DIR/roles

echo "killing: pool_sv2 jd_server jd_client translator_sv2"
sudo killall pool_sv2 jd_server jd_client translator_sv2

echo "starting: pool_sv2"
cd $ROLES_DIR/pool
screen -d -m cargo run -- -c config-examples/pool-config-local-tp-example.toml &&

echo "starting: jd-server"
cd $ROLES_DIR/jd-server
screen -d -m cargo run -- -c config-examples/jds-config-local-example.toml &&

echo "starting: jd-client"
cd $ROLES_DIR/jd-client
screen -d -m cargo run -- -c config-examples/jdc-config-local-example.toml &&

echo "starting: translator"
cd $ROLES_DIR/translator/
screen -d -m cargo run -- -c config-examples/tproxy-config-local-jdc-example.toml &&

echo "waiting for SV2 roles initialization"
sleep 10
echo "starting minerd"
minerd -a sha256d -o stratum+tcp://localhost:34255 -q -D -P
