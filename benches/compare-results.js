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

async function main() {
  const baselineSummaryFile = './baseline/results.json'
  const actualSummaryFile = './actual/results.json'
  const prUrl = process.env.PR_URL
  const githubToken = process.env.GITHUB_TOKEN

  const baselineSummary = await getK6TestResult(baselineSummaryFile)
  const actualSummary = await getK6TestResult(actualSummaryFile)

  const comparisons = {}
  let regressionDetected = false // flag to track if a regression has been detected
  for (const metricName in baselineSummary.metrics) {
    const baselineMetric = baselineSummary.metrics[metricName]
    const actualMetric = actualSummary.metrics[metricName]
    if (baselineMetric && actualMetric) {
      const percentageDifference =
        ((actualMetric.avg - baselineMetric.avg) / baselineMetric.avg) * 100

      if (percentageDifference > 5) {
        regressionDetected = true // if regression >5%, set the flag to true
      }

      comparisons[metricName] = {
        baseline: baselineMetric,
        actual: actualMetric,
        improvement: percentageDifference > 0 ? 'Yes' : 'No',
        percentageDifference: percentageDifference,
      }
    }
  }

  const originalComment = formatK6OutputForComment(actualSummary)

  let comparisonComment = '## Comparison with baseline\n'
  for (const metricName in comparisons) {
    const comparison = comparisons[metricName]
    comparisonComment += `### ${metricName}\n`
    comparisonComment += `Baseline: ${comparison.baseline.avg}\n`
    comparisonComment += `Actual: ${comparison.actual.avg}\n`
    comparisonComment += `Improvement: ${comparison.improvement}\n`
    comparisonComment += `Performance Change: ${comparison.percentageDifference.toFixed(
      2
    )}%\n\n`
  }

  // Save comparisons to a JSON file
  fs.writeFileSync('./comparisons.json', JSON.stringify(comparisons, null, 2))

  const comment = originalComment + comparisonComment

  await postCommentToPR(comment, prUrl, githubToken)

  // After posting the comment, if a regression was detected, exit with an error
  if (regressionDetected) {
    console.error('Performance regression >5% detected.')
    process.exit(1)
  }
}

main().catch(console.error)
