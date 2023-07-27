const fs = require('fs')
const path = require('path')

function formatK6OutputForComment(k6Summary) {
  let comment = '## K6 Test Results\n\n'

  for (const metricName in k6Summary.metrics) {
    const metric = k6Summary.metrics[metricName]
    comment += `### ${metricName}\n`
    comment += `Average: ${metric.avg}\n`
    comment += `Min: ${metric.min}\n`
    comment += `Max: ${metric.max}\n\n`
  }

  return comment
}

async function savePerformanceRatioToFile(comparisons, outputFilePath) {
  // Create an object to hold the performance ratio data
  const performanceRatio = {}

  // Loop through each comparison and extract the relevant data
  for (const metricName in comparisons) {
    const comparison = comparisons[metricName]
    const metricData = {
      dummyAsControl: comparison.dummyAsControl.avg,
      actual: comparison.actual.avg,
      improvement: comparison.improvement,
      percentageDifference: comparison.percentageDifference.toFixed(2),
    }

    performanceRatio[metricName] = metricData
  }

  // Write the performance ratio data to a JSON file
  fs.writeFileSync(outputFilePath, JSON.stringify(performanceRatio, null, 2))

  console.log(`Performance ratio data saved to: ${outputFilePath}`)
}

async function getK6TestResult(file) {
  const raw = fs.readFileSync(path.join(__dirname, file))
  return JSON.parse(raw)
}

async function postCommentToPR(comment, prUrl, githubToken) {
  const response = await fetch(prUrl, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${githubToken}`,
    },
    body: JSON.stringify({ body: comment }),
  })

  if (response.ok) {
    console.log('Successfully posted comment to PR.')
  } else {
    console.log('Failed to post comment to PR.')
  }
}

async function calculatePerformanceRatio(dummyAsControlMetrics, actualMetrics) {
  const comparisons = {}
  let regressionDetected = false // flag to track if a regression has been detected

  dummyAsControlMetrics.map((metric) => {
    const metricName = Object.keys(metric)
    const dummyAsControlMetric = metric[metricName].values
    const actualMetric = metric[metricName].values

    if (
      dummyAsControlMetric &&
      actualMetric &&
      'avg' in dummyAsControlMetric &&
      'avg' in actualMetric
    ) {
      const percentageDifference =
        ((actualMetric.avg - dummyAsControlMetric.avg) /
          dummyAsControlMetric.avg) *
        100

      if (percentageDifference > 5) {
        regressionDetected = true // if regression >5%, set the flag to true
      }

      comparisons[metricName] = {
        dummyAsControl: dummyAsControlMetric,
        actual: actualMetric,
        improvement: percentageDifference > 0 ? 'Yes' : 'No',
        percentageDifference: percentageDifference,
      }
    } else {
      // Handle the case where a metric exists in one file but not the other or doesn't have an avg property
      console.log(
        `Metric '${metricName}' not found in both dummyAsControl and actual results or missing 'avg' property.`
      )
    }
  })

  return { comparisons, regressionDetected }
}

async function main() {
  const dummyAsControlSummaryFile = './dummy-control/results.json'
  const actualSummaryFile = './actual/results.json'
  const prUrl = process.env.PR_URL
  const githubToken = process.env.GITHUB_TOKEN

  const dummyAsControlSummary = await getK6TestResult(dummyAsControlSummaryFile)
  const actualSummary = await getK6TestResult(actualSummaryFile)

  const { comparisons, regressionDetected } = await calculatePerformanceRatio(
    dummyAsControlSummary,
    actualSummary
  )

  const originalComment = formatK6OutputForComment(actualSummary)

  let comparisonComment = '## Comparison with dummyAsControl\n'
  for (const metricName in comparisons) {
    const comparison = comparisons[metricName]
    comparisonComment += `### ${metricName}\n`
    comparisonComment += `dummyAsControl: ${comparison.dummyAsControl.avg}\n`
    comparisonComment += `Actual: ${comparison.actual.avg}\n`
    comparisonComment += `Improvement: ${comparison.improvement}\n`
    comparisonComment += `Performance Change: ${comparison.percentageDifference.toFixed(
      2
    )}%\n\n`
  }

  const outputFilePath = './benches/performance_ratio.json'
  await savePerformanceRatioToFile(comparisons, outputFilePath)

  const comment = originalComment + comparisonComment

  await postCommentToPR(comment, prUrl, githubToken)

  // After posting the comment, if a regression was detected, exit with an error
  if (regressionDetected) {
    console.error('Performance regression >5% detected.')
    process.exit(1)
  }
}

main().catch(console.error)
