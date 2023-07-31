#!/bin/bash

function get_cpu_usage {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        echo $(grep 'cpu ' /proc/stat | awk '{usage=($2+$4)*100/($2+$4+$5)} END {print usage}')
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo $(top -l 1 | awk '/CPU usage/ {print $3}' | cut -d'%' -f1)
    fi
}

CPU_USAGE_THRESHOLD=5.0
INITIAL_CPU_USAGE=$(get_cpu_usage)
CPU_USAGE_LIMIT=$(echo "$INITIAL_CPU_USAGE + $CPU_USAGE_THRESHOLD" | bc)

# cooldown to ensure both K6 benchmarks have fair CPU usage to utilize
function cool_down_till_initial_cpu_usage {
    echo "CURRENT CPU USAGE IS: $(get_cpu_usage)"

    echo "Starting cooldown..."
    MAX_WAIT_TIME=420  # Maximum wait time of 7 minutes
    START_TIME=$(date +%s)  # Get the current time

    # This loop will check the CPU usage
    while true; do
        CURRENT_TIME=$(date +%s)  # Get the current time
        ELAPSED_TIME=$(($CURRENT_TIME - $START_TIME))

        # If the maximum wait time has been reached, exit the loop
        if [ $ELAPSED_TIME -ge $MAX_WAIT_TIME ]; then
            echo "Maximum wait time reached. Proceeding to the next test..."
            break
        fi


        CPU_USAGE=$(get_cpu_usage)  # Get the current CPU usage


        # If the CPU usage is below the limit, exit the loop
        if (( $(echo "$CPU_USAGE < $CPU_USAGE_LIMIT" | bc -l) )); then
            break
        fi

        # Log the current CPU usage
        echo "CURRENT CPU USAGE: ${CPU_USAGE}%"
        echo "Waiting for cooldown. It should be ${CPU_USAGE_LIMIT}% or less to start the next test."

        # If the CPU usage is above the threshold, wait for 5 seconds before checking again
        sleep 5
    done
    echo "Cooldown completed."
    echo "CPU USAGE AFTER COOLDOWN: ${CPU_USAGE}"
}

# cleanup on exit
function cleanup_conductor_bench {
    echo "Stopping Conductor and its source servers..."
    
    # Array of process IDs
    pids=("$SOURCE_SERVER_PID" "$SERVER_PID")

    # Loop over the process IDs
    for pid in "${pids[@]}"; do
        # Check if the process is running before killing it
        if [ -n "$pid" ] && ps -p "$pid" > /dev/null; then
            kill "$pid"
        fi
    done
}

function cleanup_dummy_server {
    echo "Stopping the dummy server..."

    # Check if the DUMMY_CONTROL_SERVER_PID process is running before killing it
    if [ -n "$DUMMY_CONTROL_SERVER_PID" ] && ps -p "$DUMMY_CONTROL_SERVER_PID" > /dev/null; then
        kill "$DUMMY_CONTROL_SERVER_PID"
    fi

}

function check_if_server_is_running {
    echo "Checking $2 availability..."
    for i in {1..10}
    do
        curl -s http://localhost:$1/graphql > /dev/null
        if [ $? -eq 0 ]; then
            echo "Conductor $2 is up and running!"
            break
        fi
        sleep 1
    done
}

function cleanup_all_servers {
    cleanup_conductor_bench
    cleanup_dummy_server
    exit 0
}

# Handle interrupt signal (e.g., CTRL+C) to stop the servers gracefully
trap cleanup_all_servers EXIT SIGINT SIGTERM

# Building source server binary in release mode
echo "Building Source Server for Gateway project..."
cd ./benches/actual/source_server && cargo build --release && cd ../../..

# Building Conductor binary in release mode
echo "Building Conductor Gateway project..."
cargo build --release

# Building Baseline server binary in release mode
echo "Building the Baseline Server project..."
cd benches/dummy_control/dummy_server && cargo build --release && cd ../../..

# Starting the baseline server
echo "Starting the Baseline server..."
./benches/dummy_control/dummy_server/target/release/baseline_server &
# Saving the PID of the baseline server process
DUMMY_CONTROL_SERVER_PID=$!

check_if_server_is_running 8001 "Baseline Server"

cool_down_till_initial_cpu_usage

# Running K6 test for the Dummy baseline
echo "Running K6 test on the dummy as control baseline..."
k6 run ./benches/dummy_control/k6.js

cleanup_dummy_server

# Starting the server
echo "Starting the Source server..."
./benches/actual/source_server/target/release/source-server &
# Saving the PID of the server process
SOURCE_SERVER_PID=$!

check_if_server_is_running 4000 "Source Server"

# Starting the server
echo "Starting the Conductor server..."
./target/release/conductor ./benches/config.yaml &
# Saving the PID of the server process
SERVER_PID=$!

check_if_server_is_running 8000 "Conductor Server"

cool_down_till_initial_cpu_usage

# Running K6 test
echo "Running K6 test on the Conductor server..."
k6 run ./benches/actual/k6.js

# Run the JavaScript script for result comparison and printing
npx ts-node ./benches/compare-results.ts
status=$?

# if the command failed (status != 0), cleanup and exit with the same status
if [ $status -ne 0 ]; then
  echo "Error running compare script, performing cleanup..."
  cleanup_conductor_bench
  exit $status
fi
# Stop the servers
cleanup_conductor_bench
exit 0