import { execSync } from 'child_process'
import fs from 'fs'
import path from 'path'

interface SummaryJSON {
  success_rate: number
  duration: {
    max: number
    'p(90)': number
    'p(95)': number
    avg: number
    min: number
    med: number
  }
}

type ValueOf<T> = T[keyof T]
type SummaryJSONValueType = ValueOf<SummaryJSON>
interface MetricComparisonRatio {
  dummyAsControl: SummaryJSONValueType
  actual: SummaryJSONValueType
  ratio: number | { [key in keyof SummaryJSON['duration']]: number }
  perfDiffInPerc: number | { [key in keyof SummaryJSON['duration']]: number }
}

interface ComparisonRatios {
  success_rate: MetricComparisonRatio
  duration: MetricComparisonRatio
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

function didImproveOrRegress(
  comparison: MetricComparisonRatio,
  isImprovement: boolean
): boolean {
  if (typeof comparison.perfDiffInPerc === 'number') {
    return isImprovement
      ? comparison.perfDiffInPerc > 0
      : comparison.perfDiffInPerc < 0
  } else {
    return Object.values(comparison.perfDiffInPerc).some((val) =>
      isImprovement ? val > 0 : val < 0
    )
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

function calculatePerformanceRatio(
  dummyAsControlMetrics: SummaryJSON,
  actualMetrics: SummaryJSON
) {
  const comparisons: Partial<ComparisonRatios> = {}

  const metricNames = ['success_rate', 'duration'] as const

  for (let metricName of metricNames) {
    const dummyAsControlMetric = dummyAsControlMetrics[metricName]
    const actualMetric = actualMetrics[metricName]

    if (dummyAsControlMetric && actualMetric) {
      if (metricName !== 'success_rate') {
        const comparison: MetricComparisonRatio = {
          dummyAsControl: dummyAsControlMetrics[metricName],
          actual: actualMetrics[metricName],
          ratio: {} as any,
          perfDiffInPerc: {} as any,
        }

        for (let nestedMetricName in actualMetrics.duration) {
          const dummyAsControlNestedMetric =
            dummyAsControlMetrics.duration[
              nestedMetricName as keyof typeof dummyAsControlMetrics.duration
            ]
          const actualNestedMetric =
            actualMetrics.duration[
              nestedMetricName as keyof typeof actualMetrics.duration
            ]

          const newRatio = dummyAsControlNestedMetric / actualNestedMetric
          let differencePercentage = 0

          if (prevRatioFileExists) {
            const oldRatio = prevPerfRatio[metricName].ratio[nestedMetricName]

            differencePercentage = ((newRatio - oldRatio) / newRatio) * 100
          }

          // @ts-ignore
          comparison.ratio[nestedMetricName] = newRatio
          if (
            // @ts-ignore
            Math.abs(differencePercentage) > THRESHOLD_PERCENTAGE
          )
            // @ts-ignore
            comparison.perfDiffInPerc[nestedMetricName] = differencePercentage
        }

        comparisons[metricName] = comparison
      } else {
        // @ts-ignore
        const newRatio = dummyAsControlMetric / actualMetric
        let differencePercentage = 0

        if (prevRatioFileExists) {
          const oldRatio = prevPerfRatio[metricName].ratio

          // higher is better -- multiply by -1 to reverse the sign
          differencePercentage = -1 * ((newRatio - oldRatio) / newRatio) * 100
        }

        comparisons[metricName] = {
          // @ts-ignore
          dummyAsControl: dummyAsControlMetric,
          // @ts-ignore
          actual: actualMetric,
          ratio: newRatio,
          perfDiffInPerc:
            Math.abs(differencePercentage) > THRESHOLD_PERCENTAGE
              ? differencePercentage
              : 0,
        }
      }
    } else {
      console.warn(
        `Metric '${metricName}' not found in both dummyAsControl and actual results.`
      )
    }
  }

  return comparisons as ComparisonRatios
}

// Save performance ratio and comments to file
function savePerformanceDataToFile(
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

  const comparisons = calculatePerformanceRatio(
    dummyAsControlSummary,
    actualSummary
  )

  let comment = '## 🧪 K6 Performance Results (Lower is Better)\n\n'

  console.log(comparisons)
  for (const metricName in comparisons) {
    // @ts-ignore
    const comparison = comparisons[metricName]

    if (
      didImproveOrRegress(comparison, true) ||
      didImproveOrRegress(comparison, false)
    ) {
      const didImprove =
        typeof comparison.perfDiffInPerc === 'number'
          ? comparison.perfDiffInPerc
          : Object.values(comparison.perfDiffInPerc).some((e: any) => e > 0)
      const changePercentage =
        typeof comparison.perfDiffInPerc === 'number'
          ? comparison.perfDiffInPerc
          : (Object.values(comparison.perfDiffInPerc)[0] as any).toFixed(2)

      const template = `### ${didImprove ? '🚀' : '❌'} ${metricName} - ${
        didImprove ? 'Improved' : 'Regressed'
      } by ${changePercentage}%\n\n\`\`\`json\n${JSON.stringify(
        actualSummary,
        null,
        2
      )}\n\`\`\``

      comment += template
    } else {
      comment += `\n### 🧘‍♂️ ${metricName} - Stable!\n\n\`\`\`json\n${JSON.stringify(
        actualSummary,
        null,
        2
      )}\n\`\`\`\n`
    }
  }

  const IS_GITHUB_CI = process.env.GITHUB_ACTIONS

  if (IS_GITHUB_CI) {
    await postCommentToPR(comment, prUrl!, githubToken!)
  } else {
    fs.writeFileSync('./bench-report.md', comment)
  }

  const didImprov = Object.values(comparisons).some((e) =>
    didImproveOrRegress(e, true)
  )
  const didRegress = Object.values(comparisons).some((e) =>
    didImproveOrRegress(e, false)
  )

  const didImproveWithNoRegressions = didImprov && !didRegress

  // Save performance data to file if there was at least one improvement or if ratio file doesn't exist
  if (!prevRatioFileExists || didImproveWithNoRegressions) {
    savePerformanceDataToFile(comparisons, outputFilePath)

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
      // @ts-ignore
      (key) => comparisons[key].perfDiffInPerc < 0
    )
    throw new Error(
      `Performance regressed in ${regressedMetrics.join(', ')} metric(s)!`
    )
  }
}

// Execute main function and handle any potential errors
main().catch(console.error)
