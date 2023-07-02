#!/bin/bash

# Start the baseline benchmark (simple Node.js server)
node benches/baseline.js &

# Save the PID of the baseline server process
BASELINE_PID=$!

# Wait for the baseline server to be available
sleep 2

# Run the current benchmark (real test) using K6 and output metrics to a JSON file
k6 run benches/benchmark.js --out json=result.json

# Get the performance metrics of the baseline and current benchmarks
BASELINE_VALUE=$(curl -s http://localhost:8001/metrics | grep -oP 'metric_name \K[0-9.]+')
CURRENT_VALUE=$(cat result.json | jq -r '.metrics."metric_name"')

# Calculate the performance ratio
RATIO=$(awk "BEGIN {print $CURRENT_VALUE / $BASELINE_VALUE}")

# Compare the performance ratio against the threshold (e.g., 20%)
THRESHOLD=0.80
if (( $(echo "$RATIO < $THRESHOLD" | bc -l) )); then
  echo "Performance regression detected!"
else
  echo "Performance improvement or within the safe range."
fi

# Stop the baseline server
kill $BASELINE_PID
