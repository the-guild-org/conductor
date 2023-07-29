const { execSync } = require('child_process')
const fs = require('fs')
const path = require('path')

const THRESHOLD_PERCENTAGE = 5
const prevRatioFilePath = path.join(__dirname, 'performance_ratio.json')
const prevRatioFileExists = fs.existsSync(prevRatioFilePath)
const prevPerfRatio = prevRatioFileExists
  ? JSON.parse(fs.readFileSync(prevRatioFilePath))
  : null

function extractInfoFromPrUrl(prUrl) {
  const urlSegments = prUrl.split('/')
  return {
    owner: urlSegments[3],
    repo: urlSegments[4],
    issueNumber: urlSegments[6],
  }
}

async function postCommentToPR(comment, prUrl, githubToken) {
  const { owner, repo, issueNumber } = extractInfoFromPrUrl(prUrl)

  const commentsUrl = `https://api.github.com/repos/${owner}/${repo}/issues/${issueNumber}/comments`

  const existingCommentsResponse = await fetch(commentsUrl, {
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${githubToken}`,
    },
  })

  const existingComments = await existingCommentsResponse.json()

  // Find the bot comment if it exists
  const botComment = existingComments.find(
    (c) => c.user.login === 'github-actions[bot]'
  )

  // If the bot comment exists, update it. Otherwise, create a new comment.
  if (botComment) {
    const response = await fetch(botComment.url, {
      method: 'PATCH',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${githubToken}`,
      },
      body: JSON.stringify({ body: comment }),
    })

    if (response.ok) {
      console.log('Successfully updated existing comment.')
    } else {
      console.log('Failed to update existing comment.')
    }
  } else {
    const response = await fetch(commentsUrl, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${githubToken}`,
      },
      body: JSON.stringify({ body: comment }),
    })

    if (response.ok) {
      console.log('Successfully posted new comment.')
    } else {
      console.log('Failed to post new comment.')
    }
  }
}

// Calculate the performance ratio between actual and control metrics
async function calculatePerformanceRatio(dummyAsControlMetrics, actualMetrics) {
  const comparisons = {}

  dummyAsControlMetrics.map((metric) => {
    const metricName = Object.keys(metric)[0]

    const dummyAsControlMetric = metric[metricName].values
    const actualMetric = actualMetrics.find(
      (e) => Object.keys(e)[0] === metricName
    )[metricName].values

    if (dummyAsControlMetric && actualMetric) {
      const newRatio = dummyAsControlMetric.avg / actualMetric.avg
      let differencePercentage = 0

      if (prevRatioFileExists) {
        const oldRatio = prevPerfRatio[metricName].ratio
        differencePercentage =
          (Math.abs(newRatio - oldRatio) / ((newRatio + oldRatio) / 2)) * 100
      }

      comparisons[metricName] = {
        dummyAsControl: dummyAsControlMetric,
        actual: actualMetric,
        ratio: newRatio,
        diffPercentage: differencePercentage,
        didImprove: differencePercentage > 0,
      }
    } else {
      console.log(
        `Metric '${metricName}' not found in both dummyAsControl and actual results or missing 'avg' property.`
      )
    }
  })

  return comparisons
}
// Save performance ratio and comments to file
async function savePerformanceDataToFile(comparisons, outputFilePath) {
  // Write the performance ratio data to a JSON file
  fs.writeFileSync(outputFilePath, JSON.stringify(comparisons, null, 2))
  console.log(`Performance ratio data has been saved to: ${outputFilePath}`)
}

// Fetch k6 test results from JSON file
async function getK6TestResult(file) {
  const raw = fs.readFileSync(path.join(__dirname, file))
  return JSON.parse(raw)
}

async function main() {
  const dummyAsControlSummaryFile = './dummy_control/results.json'
  const actualSummaryFile = './actual/results.json'
  const outputFilePath = './benches/performance_ratio.json'
  const prUrl = process.env.PR_URL
  const githubToken = process.env.GITHUB_TOKEN

  const dummyAsControlSummary = await getK6TestResult(dummyAsControlSummaryFile)
  const actualSummary = await getK6TestResult(actualSummaryFile)

  const comparisons = await calculatePerformanceRatio(
    dummyAsControlSummary,
    actualSummary
  )

  let improvements = []
  let regressions = []
  let stable = []

  for (const metricName in comparisons) {
    const comparison = comparisons[metricName]

    if (Math.abs(comparison.diffPercentage) > THRESHOLD_PERCENTAGE) {
      const changePercentage = comparison.diffPercentage.toFixed(2)

      const template = `### ${
        comparison.didImprove ? '🚀' : '❌'
      } ${metricName} - ${
        comparison.didImprove ? 'Improved' : 'Regressed'
      } by ${Math.abs(changePercentage)}%\n\n\`\`\`diff\n+now: ${
        comparison.actual.avg
      }ms\n-previous: ${prevPerfRatio[metricName].actual.avg}ms\n\`\`\`\n`

      comparison.didImprove
        ? improvements.push(template)
        : regressions.push(template)
    } else {
      stable.push(
        `### 🧘‍♂️ ${metricName} - Stable!\n\n\`\`\`\nnow: ${
          comparison.actual.avg
        }ms\nprevious: ${
          prevPerfRatio?.[metricName]?.actual?.avg || comparison.actual.avg
        }ms\n\`\`\`\n`
      )
    }
  }
  // Sort by performance: improvements, regressions, stable
  const sortedComments = [...improvements, ...regressions, ...stable]

  // Combine all comments
  const comment =
    '## 🧪 K6 Test Results (Lower is Better)\n\n' + sortedComments.join('\n')

  const IS_GITHUB_CI = process.env.GITHUB_ACTIONS

  if (IS_GITHUB_CI) {
    await postCommentToPR(comment, prUrl, githubToken)
  } else {
    fs.writeFileSync('./bench-report.md', comment)
  }

  // Save performance data to file if there was at least one improvement or if ratio file doesn't exist
  if (
    !prevRatioFileExists ||
    (improvements.length > 0 && regressions.length === 0)
  ) {
    await savePerformanceDataToFile(comparisons, outputFilePath)

    if (IS_GITHUB_CI) {
      // Run Git commands
      const gitAddCmd = `git add ${outputFilePath}`
      const gitCommitCmd = 'git commit -m "Update performance ratio file"'
      const gitPushCmd = 'git push'

      try {
        console.log('Committing and pushing changes...')
        execSync(gitAddCmd)
        execSync(gitCommitCmd)
        execSync(gitPushCmd)
        console.log('Changes pushed successfully')
      } catch (error) {
        console.error('Failed to push changes:', error)
      }
    }
  }

  // Fail the CI process if any regressions were detected
  if (regressions.length > 0) {
    throw new Error(`Performance regressed in ${regressions.length} metric(s)!`)
  }
}

// Execute main function and handle any potential errors
main().catch(console.error)
