#!/bin/bash

# Function to cleanup on exit
function cleanup {
    echo "Stopping the servers..."
    kill $SERVER_PID
    kill $BASELINE_SERVER_PID
    kill $SOURCE_SERVER_PID
    exit 0
}

# Handle interrupt signal (e.g., CTRL+C) to stop the servers gracefully
trap cleanup EXIT SIGINT SIGTERM

# Start the source server
echo "Starting the source server..."
node ./benches/conductor_source_server.js &
# Save the PID of the source server process
SOURCE_SERVER_PID=$!

# Check source server availability
echo "Checking source server availability..."
for i in {1..10}
do
    curl -s http://localhost:4000/graphql > /dev/null
    if [ $? -eq 0 ]; then
        echo "Source server is up and running!"
        break
    fi
    sleep 1
done

# Building Conductor binary in release mode
echo "Building the Rust project..."
cargo build --release

# Starting the server
echo "Starting the Conductor server..."
./target/release/conductor ./benches/config.yaml &
# Saving the PID of the server process
SERVER_PID=$!

# Checking server availability
echo "Checking Conductor server availability..."
for i in {1..10}
do
    curl -s http://localhost:8000/graphql > /dev/null
    if [ $? -eq 0 ]; then
        echo "Conductor server is up and running!"
        break
    fi
    sleep 1
done

# Running K6 test
echo "Running K6 test on the Conductor server..."
k6 run --out json=./benches/k6-results.json ./benches/k6.js

# Starting the baseline server
echo "Starting the baseline server..."
node baseline_server.js &
# Saving the PID of the baseline server process
BASELINE_SERVER_PID=$!

# Checking baseline server availability
echo "Checking baseline server availability..."
for i in {1..10}
do
    curl -s http://localhost:8001 > /dev/null
    if [ $? -eq 0 ]; then
        echo "Baseline server is up and running!"
        break
    fi
    sleep 1
done

# Running K6 test
echo "Running K6 test on the baseline server..."
k6 run --out json=./benches/k6-baseline-results.json ./benches/k6_baseline.js

# Run the JavaScript script for result comparison and printing
node ./benches/compare-results.js ./benches/k6-results.json ./benches/k6-baseline-results.json

# Stop the servers
cleanup
