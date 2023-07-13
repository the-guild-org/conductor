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

// Read the previous ratio file
const ratioFilePath = './ratio.json'
const ratioFileExists = fs.existsSync(ratioFilePath)
const previousRatio = ratioFileExists
  ? JSON.parse(fs.readFileSync(ratioFilePath))
  : {}

// Calculate the ratio between previous and current baseline and actual code
const ratio = {}
for (const metric of metrics) {
  const actualMetric = actualResults.metrics[metric].mean

  if (baselineFileExists && baselineResults[metric]) {
    const baselineMetric = baselineResults[metric]

    // Calculate performance change percentage
    const currentRatio = actualMetric / baselineMetric
    const previousRatioMetric = previousRatio[metric] || 0

    // Calculate the ratio change percentage
    const ratioChange =
      ((currentRatio - previousRatioMetric) / previousRatioMetric) * 100

    // Format the output
    const colorCode = ratioChange > 0 ? '\x1b[32m' : '\x1b[31m'
    const ratioChangeFormatted = ratioChange.toFixed(2)

    // Print the result with color
    console.log(
      `Ratio change for ${metric}: ${colorCode}${ratioChangeFormatted}%\x1b[0m`
    )

    // Throw an error if the ratio regression is more than 5% for any metric
    if (ratioChange < -5) {
      throw new Error(
        `Ratio regression of more than 5% detected for ${metric}: ${ratioChangeFormatted}%`
      )
    }

    // Update the ratio for the metric
    ratio[metric] = currentRatio
  }

  // Update the baseline for the metric
  baselineResults[metric] = actualMetric
}

// Write the current ratio to ratio.json
fs.writeFileSync(ratioFilePath, JSON.stringify(ratio))

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
