// @ts-check

// @ts-expect-error - TS doesn't know this import
import { textSummary } from 'https://jslib.k6.io/k6-summary/0.0.1/index.js'
// @ts-expect-error - TS doesn"t know this import
import { githubComment } from 'https://raw.githubusercontent.com/dotansimha/k6-github-pr-comment/master/lib.js'
import { check } from 'k6'
import http from 'k6/http'

const VUS = 200
const DURATION = '60s'

export const options = {
  stages: [
    { duration: '10s', target: 10 },
    { duration: DURATION, target: VUS },
    { duration: '10s', target: 0 },
  ],
  thresholds: {
    http_req_duration: ['avg<=8'],
    http_req_failed: ['rate==0'],
  },
}

export function handleSummary(data) {
  if (__ENV.GITHUB_TOKEN) {
    githubComment(data, {
      token: __ENV.GITHUB_TOKEN,
      commit: __ENV.GITHUB_SHA,
      pr: __ENV.GITHUB_PR,
      org: 'the-guild-org',
      repo: 'conductor-t2',
      renderTitle({ passes }) {
        return passes ? '✅ Benchmark Results' : '❌ Benchmark Failed'
      },
      renderMessage({ passes, checks, thresholds }) {
        const result = []

        if (thresholds.failures) {
          result.push(
            `**Performance regression detected**: it seems like your Pull Request adds some extra latency to GraphQL Yoga`
          )
        }

        if (checks.failures) {
          result.push('**Failed assertions detected**')
        }

        if (!passes) {
          result.push(
            `> If the performance regression is expected, please increase the failing threshold.`
          )
        }

        return result.join('\n')
      },
    })
  }
  return {
    stdout: textSummary(data, { indent: ' ', enableColors: true }),
  }
}

export default function () {
  const res = http.post(
    'http://127.0.0.1:9000/graphql',
    JSON.stringify({
      query: /* GraphQL */ `
        query authors {
          authors {
            id
            name
            company
            books {
              id
              name
              numPages
            }
          }
        }
      `,
    }),
    {
      headers: { 'Content-Type': 'application/json' },
    }
  )

  check(res, {})
}
