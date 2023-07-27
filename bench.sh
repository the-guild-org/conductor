#!/bin/bash

# Function to cleanup on exit
function cleanup {
    echo "Stopping the servers..."
    
    # Array of process IDs
    pids=("$SOURCE_SERVER_PID" "$SERVER_PID" "$BASELINE_SERVER_PID")

    # Loop over the process IDs
    for pid in "${pids[@]}"; do
        # Check if the process is running before killing it
        if [ -n "$pid" ] && ps -p "$pid" > /dev/null; then
            kill "$pid"
        fi
    done
    
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
echo "Building Conductor Gateway project..."
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
k6 run ./benches/k6.js

# Cooldown for 10sec
echo "Cooldown for 10 seconds..."
sleep 10

# Building Baseline server binary in release mode
echo "Building the Baseline Server project..."
cd benches/baseline_server && cargo build --release && cd ../..

# Starting the baseline server
echo "Starting the Baseline server..."
./benches/baseline_server/target/release/baseline_server &
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
k6 run ./benches/k6_dummy-control.js

# Run the JavaScript script for result comparison and printing
node ./benches/compare-results.js

# Stop the servers
cleanup
