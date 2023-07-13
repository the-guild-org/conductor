const fs = require('fs')

// Read command-line arguments for result file paths
const [, , currentResultsPath, previousResultsPath] = process.argv

// Read the result files
const currentResults = JSON.parse(fs.readFileSync(currentResultsPath))
const previousResults = JSON.parse(fs.readFileSync(previousResultsPath))

// Extract the relevant metric for comparison
const currentMetric = currentResults.metrics.http_req_duration.mean
const previousMetric = previousResults.metrics.http_req_duration.mean

// Calculate performance change percentage
const performanceChange =
  ((currentMetric - previousMetric) / previousMetric) * 100

// Format the output
const colorCode = performanceChange > 0 ? '\x1b[32m' : '\x1b[31m'
const performanceChangeFormatted = performanceChange.toFixed(2)

// Print the result with color
console.log(
  `Performance change: ${colorCode}${performanceChangeFormatted}%\x1b[0m`
)

// Throw an error if the performance regression is more than 5%
if (performanceChange < -5) {
  throw new Error(
    `Performance regression of more than 5% detected: ${performanceChangeFormatted}%`
  )
}
