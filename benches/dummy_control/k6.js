import { check } from 'k6'
import http from 'k6/http'
import { Trend, Counter } from 'k6/metrics'
import ports from '../ports.js'

const VUS = 1000 // virtual users
let ErrorCount = new Counter('errors')
let SuccessCount = new Counter('successes')
let Duration = new Trend('req_duration')

export let options = {
  stages: [
    { duration: '2m', target: VUS }, // Warm up stage
    { duration: '1m', target: VUS * 3 }, // Sustained high load
  ],
}

export default function () {
  const res = http.get(`http://localhost:${ports.DUMMY_CONTROL_SERVER}`)

  // Checking status and expected response
  const isStatus200 = res.status === 200

  const isExpectedResponse = res.body === 'Hello world!'

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
    './benches/dummy_control/results.json': JSON.stringify(summary, null, 2),
  }
}
