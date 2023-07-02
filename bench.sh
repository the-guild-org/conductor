#!/bin/bash

# Function to cleanup on exit
function cleanup {
    echo "Stopping the server..."
    kill $SERVER_PID
    exit 0
}

# Handle interrupt signal (e.g., CTRL+C) to stop the server gracefully
trap cleanup EXIT SIGINT SIGTERM

# Building Conductor binary in release mode
echo "Building the Rust project..."
cargo build --release

# Starting the server
echo "Starting the server..."
./target/release/conductor ./benches/config.yaml &

# Saving the PID of the server process
SERVER_PID=$!

# Checking server availability
echo "Checking server availability..."
for i in {1..10}
do
    curl -s http://localhost:8000/graphql > /dev/null
    if [ $? -eq 0 ]; then
        echo "Server is up and running!"
        break
    fi
    sleep 1
done

# Running K6 test
echo "Running K6 test..."
k6 run --out json=./benches/k6-results.json ./benches/k6.js

# Run the JavaScript script for result comparison and printing
node ./benches/compare-results.js $CURRENT_RESULTS $PREVIOUS_RESULTS

# Stop the server
cleanup