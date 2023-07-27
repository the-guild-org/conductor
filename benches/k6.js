/* eslint-disable no-undef */
import { check, sleep } from 'k6'
import http from 'k6/http'

const VUS = 1000 // 1000 virtual users

export const options = {
  stages: [
    { duration: '1m', target: VUS / 2 }, // starting with significant load
    { duration: '2m', target: VUS }, // starting with significant load
    { duration: '1m', target: 0 }, // starting with significant load

    // { duration: '2m', target: VUS }, // starting with significant load
    // { duration: '5m', target: VUS }, // maintaining significant load
    // { duration: '2m', target: VUS * 2 }, // increasing to high load
    // { duration: '5m', target: VUS * 2 }, // maintaining high load
    // { duration: '2m', target: VUS * 4 }, // approaching extreme load
    // { duration: '5m', target: VUS * 4 }, // maintaining extreme load
    // { duration: '10m', target: 0 }, // scale down. Recovery stage.
  ],
  thresholds: {
    http_req_duration: ['p(99)<1500'], // 99% of requests should be below 1.5s
    'checks{tag:check}': ['rate>0.95'], // Check execution success should be above 95%
    http_req_blocked: ['p(99)<400'], // 99% of requests should have blocked time below 400ms
    http_req_connecting: ['p(99)<500'], // 99% of requests should have connection time below 500ms
    http_req_tls_handshaking: ['p(99)<500'], // 99% of requests should have TLS handshaking time below 500ms
    http_req_sending: ['p(99)<200'], // 99% of requests should have send time below 200ms
    http_req_waiting: ['p(99)<1500'], // 99% of requests should have wait time below 1.5s
    http_req_receiving: ['p(99)<600'], // 99% of requests should have receive time below 600ms
    iteration_duration: ['p(99)<7000'], // 99% of iterations should complete below 7s
  },
}

// Initialize an object to store the total metrics and the count for averaging
const totals = {
  http_req_duration: 0,
  http_req_blocked: 0,
  http_req_connecting: 0,
  http_req_tls_handshaking: 0,
  http_req_sending: 0,
  http_req_waiting: 0,
  http_req_receiving: 0,
  iteration_duration: 0,
  my_iterations: 0,
}

export default function () {
  let res = http.post(
    'http://localhost:4000/graphql',
    JSON.stringify({
      query: 'query { countries { id } }',
      operationName: 'countries',
    }),
    { headers: { 'Content-Type': 'application/json' } }
  )

  // Add the metrics to the totals object for averaging
  const timings = res.timings
  totals.http_req_duration += timings.duration || 0
  totals.http_req_blocked += timings.blocked || 0
  totals.http_req_connecting += timings.connecting || 0
  totals.http_req_tls_handshaking += timings.tlsHandshaking || 0
  totals.http_req_sending += timings.sending || 0
  totals.http_req_waiting += timings.waiting || 0
  totals.http_req_receiving += timings.receiving || 0
  totals.iteration_duration += __VU
  totals.my_iterations += __ITER

  check(res, {
    'status was 200': (r) => r.status == 200,
    'status was not 500': (r) => r.status != 500,
    'transaction time OK': (r) => r.timings.duration < 200,
  })

  sleep(1)
}

export function handleSummary(data) {
  // Calculate the average metrics only if there are samples
  const numSamples = data.metrics.iterations.count
  if (numSamples > 0) {
    data.metrics.http_req_duration.avg = totals.http_req_duration / numSamples
    data.metrics.http_req_blocked.avg = totals.http_req_blocked / numSamples
    data.metrics.http_req_connecting.avg =
      totals.http_req_connecting / numSamples
    data.metrics.http_req_tls_handshaking.avg =
      totals.http_req_tls_handshaking / numSamples
    data.metrics.http_req_sending.avg = totals.http_req_sending / numSamples
    data.metrics.http_req_waiting.avg = totals.http_req_waiting / numSamples
    data.metrics.http_req_receiving.avg = totals.http_req_receiving / numSamples
    data.metrics.iteration_duration.avg = totals.iteration_duration / numSamples
    data.metrics.my_iterations.avg = totals.my_iterations / numSamples
  }

  // Customize the output to show only the essential metrics
  return {
    stdout: JSON.stringify(data.metrics),
    './benches/actual/results.json': JSON.stringify(data.metrics),
  }
}
