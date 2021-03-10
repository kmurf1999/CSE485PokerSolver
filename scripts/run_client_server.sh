# set rust log to debug
RUST_LOG=debug
BASE_DIR="$HOME/Documents/CSE485PokerSolver"
CLIENT_DIR="$BASE_DIR/frontend"
SERVER_DIR="$BASE_DIR/poker_solver/server"


cd $CLIENT_DIR
nohup bash -c 'npm start' > "$BASE_DIR/client.out" &
CLIENT=$!

cd $SERVER_DIR
nohup bash -c 'RUST_LOG=debug cargo watch -x "run"' > "$BASE_DIR/server.out" &
SERVER=$!

cd $BASE_DIR
tail -f "$BASE_DIR/server.out"

wait $SERVER $CLIENT

kill -9 $SERVER
kill -9 $CLIENT
# ./target/release/server &
# CLIENT=$!
# ./target/release/client &
# SERVER=$!

# wait $CLIENT $SERVER

# # kill server
# kill -9 $(lsof -i :8080 | awk -F ' ' 'NR==2{print $2}')