import { execSync } from 'child_process'
import fs from 'fs'
import path from 'path'

interface MetricValues {
  max: number
  'p(90)': number
  'p(95)': number
  avg: number
  min: number
  med: number
}

interface SummaryJSON {
  [key: string]: {
    type: string
    contains: string
    values: MetricValues
  }
}

interface ComparisonRatios {
  [key: string]: {
    dummyAsControl: MetricValues
    actual: MetricValues
    ratio: number
    perfDiffInPerc: number
  }
}

const THRESHOLD_PERCENTAGE = 5
const prevRatioFilePath = path.join(__dirname, 'performance_ratio.json')
const prevRatioFileExists = fs.existsSync(prevRatioFilePath)
const prevPerfRatio = prevRatioFileExists
  ? JSON.parse(fs.readFileSync(prevRatioFilePath, 'utf-8'))
  : null

function extractInfoFromPrUrl(prUrl: string) {
  const urlSegments = prUrl.split('/')
  return {
    owner: urlSegments[3],
    repo: urlSegments[4],
    issueNumber: urlSegments[6],
  }
}

async function postCommentToPR(
  comment: string,
  prUrl: string,
  githubToken: string
) {
  const { owner, repo, issueNumber } = extractInfoFromPrUrl(prUrl)

  const commentsUrl = `https://api.github.com/repos/${owner}/${repo}/issues/${issueNumber}/comments`

  const existingComments = await (
    await fetch(commentsUrl, {
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${githubToken}`,
      },
    })
  ).json()

  // Find the bot comment if it exists
  const botComment = existingComments.find(
    (c: any) => c.user.login === 'github-actions[bot]'
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

async function calculatePerformanceRatio(
  dummyAsControlMetrics: SummaryJSON,
  actualMetrics: SummaryJSON
) {
  const comparisons: ComparisonRatios = {}

  Object.keys(dummyAsControlMetrics).map((metricName) => {
    const dummyAsControlMetric = dummyAsControlMetrics[metricName].values
    const actualMetric = actualMetrics[metricName].values

    if (dummyAsControlMetric && actualMetric) {
      const newRatio = dummyAsControlMetric.avg / actualMetric.avg
      let differencePercentage = 0

      if (prevRatioFileExists) {
        const oldRatio = prevPerfRatio[metricName].ratio

        // old: 10
        // new: 5
        // expected perf improv: 100%
        // calculation: -1 * (((5-10) / 5) * 100) = +100%

        // old: 5
        // new: 10
        // expected perf improv: -50%
        // calculation: -1 * (((10-5) / 10) * 100) = 50%

        // we multiply by `-1` to flip the sign, resulting in negative values becoming positive, and positive values becoming negative
        // because "Lower Is Better" in all of these benchmarking metrics.
        differencePercentage = -1 * ((newRatio - oldRatio) / newRatio) * 100
      }

      comparisons[metricName] = {
        dummyAsControl: dummyAsControlMetric,
        actual: actualMetric,
        ratio: newRatio,
        perfDiffInPerc: differencePercentage,
      }
    } else {
      console.warn(
        `Metric '${metricName}' not found in both dummyAsControl and actual results or missing 'avg' property.`
      )
    }
  })

  return comparisons
}
// Save performance ratio and comments to file
async function savePerformanceDataToFile(
  comparisons: ComparisonRatios,
  outputFilePath: string
) {
  fs.writeFileSync(outputFilePath, JSON.stringify(comparisons, null, 2))
  console.log(`Performance ratio data has been saved to: ${outputFilePath}`)
}

function getK6TestResult(filePath: string): SummaryJSON {
  const raw = fs.readFileSync(path.join(__dirname, filePath), 'utf-8')
  return JSON.parse(raw)
}

async function main() {
  const dummyAsControlSummaryFile = './dummy_control/results.json'
  const actualSummaryFile = './actual/results.json'
  const outputFilePath = './benches/performance_ratio.json'
  const prUrl = process.env.PR_URL
  const githubToken = process.env.GITHUB_TOKEN

  const dummyAsControlSummary = getK6TestResult(dummyAsControlSummaryFile)
  const actualSummary = getK6TestResult(actualSummaryFile)

  const comparisons = await calculatePerformanceRatio(
    dummyAsControlSummary,
    actualSummary
  )

  let comment = '## 🧪 K6 Performance Results (Lower is Better)\n\n'

  for (const metricName in comparisons) {
    const comparison = comparisons[metricName]

    if (Math.abs(comparison.perfDiffInPerc) > THRESHOLD_PERCENTAGE) {
      const didImprove = comparison.perfDiffInPerc > 0
      const changePercentage = comparison.perfDiffInPerc.toFixed(2)

      const template = `### ${didImprove ? '🚀' : '❌'} ${metricName} - ${
        didImprove ? 'Improved' : 'Regressed'
      } by ${changePercentage}%\n\n\`\`\`diff\n+now: ${
        comparison.actual.avg
      }ms\n-previous: ${prevPerfRatio[metricName].actual.avg}ms\n\`\`\`\n`

      comment += template
    } else {
      comment += `### 🧘‍♂️ ${metricName} - Stable!\n\n`
    }
  }

  const IS_GITHUB_CI = process.env.GITHUB_ACTIONS

  if (IS_GITHUB_CI) {
    await postCommentToPR(comment, prUrl!, githubToken!)
  } else {
    fs.writeFileSync('./bench-report.md', comment)
  }

  const didImprov = Object.values(comparisons).every(
    (e) => e.perfDiffInPerc > 0
  )
  const didRegress = Object.values(comparisons).every(
    (e) => e.perfDiffInPerc < 0
  )
  const didImproveWithNoRegressions = didImprov && !didRegress

  // Save performance data to file if there was at least one improvement or if ratio file doesn't exist
  if (!prevRatioFileExists || didImproveWithNoRegressions) {
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
  if (didRegress) {
    const regressedMetrics = Object.keys(comparisons).filter(
      (key) => comparisons[key].perfDiffInPerc < 0
    )
    throw new Error(
      `Performance regressed in ${regressedMetrics.join(', ')} metric(s)!`
    )
  }
}

// Execute main function and handle any potential errors
main().catch(console.error)
