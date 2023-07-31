import { check } from 'k6'
import http from 'k6/http'
import { Trend, Counter } from 'k6/metrics'
import ports from '../ports.js'

const VUS = 1000 // virtual users
let ErrorCount = new Counter('errors')
let SuccessCount = new Counter('successes')
let Duration = new Trend('req_duration')

// Define different operations with expected results
const operations = [
  {
    name: 'countries-partial-details',
    query: 'query { countries { id name code language } }',
    expected: {
      data: {
        countries: [
          {
            id: 1,
            name: 'United States',
            code: 'US',
            language: 'English',
          },
          {
            id: 2,
            name: 'Canada',
            code: 'CA',
            language: 'English, French',
          },
          {
            id: 3,
            name: 'Portugal',
            code: 'PT',
            language: 'Portuguese',
          },
        ],
      },
    },
  },
  {
    name: 'country-full-details',
    query:
      '{ country(code: "PT") { name code language avgWage foundationDate popularDishes { name id ingredients } } }',
    expected: {
      data: {
        country: {
          name: 'Portugal',
          code: 'PT',
          language: 'Portuguese',
          avgWage: 18000,
          foundationDate: '1143-10-05',
          popularDishes: [
            {
              name: 'Pastel de nata',
              id: 3,
              ingredients: ['Egg', 'Sugar', 'Cream'],
            },
          ],
        },
      },
    },
  },
  {
    name: 'country-partial-details',
    query: `query { country(code: "US") { id name foundationDate popularDishes { name id ingredients } } }`,
    expected: {
      data: {
        country: {
          id: 1,
          name: 'United States',
          foundationDate: '1776-07-04',
          popularDishes: [
            {
              name: 'Hamburger',
              id: 1,
              ingredients: ['Bread', 'Cheese', 'Ham'],
            },
          ],
        },
      },
    },
  },
]

export let options = {
  stages: [
    { duration: '2m', target: VUS }, // Warm up stage
    { duration: '1m', target: VUS * 3 }, // Sustained high load
  ],
}

export default function () {
  // Randomly select an operation
  const operation = operations[Math.floor(Math.random() * operations.length)]

  const res = http.post(
    `http://localhost:${ports.CONDUCTOR}/graphql`,
    JSON.stringify({
      query: operation.query,
      operationName: operation.name,
    }),
    { headers: { 'Content-Type': 'application/json' } }
  )

  // Checking status and expected response
  const isStatus200 = res.status === 200
  let responseBody = {}
  try {
    responseBody = JSON.parse(res.body)
  } catch (error) {
    console.log('Failed to parse response:', error)
  }
  const isExpectedResponse =
    JSON.stringify(responseBody) === JSON.stringify(operation.expected)

  if (isStatus200 && isExpectedResponse) {
    SuccessCount.add(1)
  } else {
    ErrorCount.add(1)
  }

  check(res, {
    'is status 200': () => isStatus200,
    'is expected response': () => isExpectedResponse,
  })

  Duration.add(res.timings.duration)
}

export function handleSummary(data) {
  const successCount = data.metrics.successes
    ? data.metrics.successes.values.count
    : 0
  const errorCount = data.metrics.errors ? data.metrics.errors.values.count : 0
  const summary = {
    success_rate: successCount / (errorCount + successCount),
    duration: data.metrics.http_req_duration.values,
  }

  // Customize the output to show only the essential metrics
  return {
    stdout: JSON.stringify(summary, null, 2),
    './benches/actual/results.json': JSON.stringify(summary, null, 2),
  }
}
