const fs = require('fs')
const path = require('path')
const readline = require('readline')

// Read command-line arguments for result file paths
const [, , actualResultsPath, baselineFilePath] = process.argv

// Read the result files
const actualResults = JSON.parse(fs.readFileSync(actualResultsPath))
const baselineFileExists = fs.existsSync(baselineFilePath)
const baselineResults = baselineFileExists
  ? JSON.parse(fs.readFileSync(baselineFilePath))
  : {}

// Relevant metrics for comparison
const metrics = JSON.parse(
  fs.readFileSync(path.join(__dirname, 'metrics.json'), 'utf-8')
)

// Compare each metric
for (const metric of metrics) {
  const actualMetric = actualResults.metrics[metric].mean

  if (baselineFileExists && baselineResults[metric]) {
    const baselineMetric = baselineResults[metric]

    // Calculate performance change percentage
    const performanceChange =
      ((actualMetric - baselineMetric) / baselineMetric) * 100

    // Format the output
    const colorCode = performanceChange > 0 ? '\x1b[32m' : '\x1b[31m'
    const performanceChangeFormatted = performanceChange.toFixed(2)

    // Print the result with color
    console.log(
      `Performance change for ${metric}: ${colorCode}${performanceChangeFormatted}%\x1b[0m`
    )

    // Throw an error if the performance regression is more than 5% for any metric
    if (performanceChange < -5) {
      throw new Error(
        `Performance regression of more than 5% detected for ${metric}: ${performanceChangeFormatted}%`
      )
    }
  }

  // Update the baseline for the metric
  baselineResults[metric] = actualMetric
}

// Prompt the user to update the baseline, if not in a CI environment
if (!process.env.CI) {
  // Initialize the readline interface
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
  })

  rl.question('Do you want to update the baseline? (Y/N) ', (answer) => {
    if (answer.toLowerCase() === 'y') {
      fs.writeFileSync(baselineFilePath, JSON.stringify(baselineResults))
      console.log('Baseline updated.')
    }
    rl.close()
  })
} else if (process.env.CI === 'true') {
  // In a CI environment, we fail the build if there was a performance regression,
  // but we don't update the baseline
  process.exit(1)
}
