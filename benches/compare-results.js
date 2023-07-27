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

const THRESHOLD_PERCENTAGE = 5

async function savePerformanceRatioToFile(comparisons, outputFilePath) {
  let performanceImproved = false
  let performanceStable = false
  let performanceRegression = false

  // Check if performance has improved, regressed, or stayed the same
  for (const metricName in comparisons) {
    const percentageDifference = comparisons[metricName].percentageDifference
    if (percentageDifference > THRESHOLD_PERCENTAGE) {
      performanceRegression = true
    } else if (percentageDifference < -THRESHOLD_PERCENTAGE) {
      performanceImproved = true
    } else {
      performanceStable = true
    }
  }

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

  const prevRatioFilePath = path.join(__dirname, 'performance_ratio.json')
  const isFirstRun = !fs.existsSync(prevRatioFilePath)

  // Write the performance ratio data to a JSON file
  fs.writeFileSync(outputFilePath, JSON.stringify(performanceRatio, null, 2))
  console.log(`Performance ratio data has been saved to: ${outputFilePath}`)

  if (isFirstRun) {
    console.log('This is the initial performance data.')
  } else {
    if (performanceImproved && !performanceRegression) {
      console.log('Good Job! Performance Improved!')
    } else if (performanceRegression && !performanceImproved) {
      console.log('Performance Regression Detected!')
    } else if (performanceStable) {
      console.log('No Significant Change in Performance.')
    }
  }
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
    const metricName = Object.keys(metric)[0]

    const dummyAsControlMetric = metric[metricName].values
    const actualMetric = actualMetrics.find(
      (e) => Object.keys(e)[0] === metricName
    )[metricName].values

    if (dummyAsControlMetric && actualMetric) {
      const percentageDifference =
        ((actualMetric.avg - dummyAsControlMetric.avg) /
          dummyAsControlMetric.avg) *
        100

      const prevRatioFilePath = path.join(__dirname, 'performance_ratio.json')
      if (fs.existsSync(prevRatioFilePath)) {
        const prevPerfRatio = JSON.parse(fs.readFileSync(prevRatioFilePath))

        const perfDiff =
          prevPerfRatio[metricName].percentageDifference - percentageDifference

        if (perfDiff > 5) {
          regressionDetected = true // if regression >5%, set the flag to true
        }
      }

      comparisons[metricName] = {
        dummyAsControl: dummyAsControlMetric,
        actual: actualMetric,
        improvement: percentageDifference < 0 ? 'Yes' : 'No',
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

    // Calculate the percentage difference and add it to the comment
    const absolutePercentageDifference = Math.abs(
      comparison.percentageDifference
    )

    if (absolutePercentageDifference > THRESHOLD_PERCENTAGE) {
      if (comparison.improvement === 'Yes') {
        const changePercentage = comparison.percentageDifference.toFixed(2)
        comparisonComment += `Performance Improved by: ${Math.abs(
          changePercentage
        )}%\n`
      } else if (comparison.improvement === 'No') {
        const changePercentage = Math.abs(
          comparison.percentageDifference.toFixed(2)
        )
        comparisonComment += `Performance Regressed by: ${changePercentage}%\n`
      }
    } else {
      comparisonComment += `No Significant Change in Performance.\n`
    }
  }
  const comment = originalComment + comparisonComment

  console.log(comment)
  // await postCommentToPR(comment, prUrl, githubToken);

  // After posting the comment, if a regression was detected, exit with an error
  if (regressionDetected) {
    console.error('Performance regression >5% detected.')
    process.exit(1)
  } else {
    const outputFilePath = './benches/performance_ratio.json'
    await savePerformanceRatioToFile(comparisons, outputFilePath)
  }
}

main().catch(console.error)
