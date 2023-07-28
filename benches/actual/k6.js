/* eslint-disable no-undef */
import { check, sleep } from 'k6'
import http from 'k6/http'
import { Trend } from 'k6/metrics'

const VUS = 1000 // 1000 virtual users

export const options = {
  stages: [
    { duration: '1m', target: 0 }, // starting with significant load
    { duration: '2m', target: VUS }, // starting with significant load
    { duration: '5m', target: VUS }, // maintaining significant load
    { duration: '2m', target: VUS * 2 }, // increasing to high load
    { duration: '5m', target: VUS * 2 }, // maintaining high load
    { duration: '2m', target: VUS * 4 }, // approaching extreme load
    { duration: '5m', target: VUS * 4 }, // maintaining extreme load
    { duration: '10m', target: 0 }, // scale down. Recovery stage.
  ],
  thresholds: {
    /**
     * Represents the total time for the request. It includes `http_req_sending`, `http_req_waiting`, and
     * `http_req_receiving`. 99% of requests should complete within 300ms. This threshold is reasonable
     * given the simplicity and locality of the queries.
     */
    http_req_duration: ['p(99)<300'],
    /**
     * Used to track the rate of successful checks. It measures how many of the completed requests actually
     * returned the expected result. More than 99% of checks should pass.
     */
    'checks{tag:check}': ['rate>0.99'],
    /**
     * Measures the time from when a request is initiated up to the point when the actual request is started.
     * It captures time spent in OS-level queues and other delays. As we're testing locally, the blocking
     * time should be very low. Hence, 99% of requests should have blocked time below 10ms.
     */
    http_req_blocked: ['p(99)<10'],
    /**
     * Represents the time spent establishing the TCP connection to the remote host. As all tests are local,
     * this should be quite low. Thus, 99% of requests should establish a connection in under 10ms.
     */
    http_req_connecting: ['p(99)<10'],
    /**
     * Tracks the time spent on the TLS handshake when establishing a secure connection. As everything is
     * local and there's no SSL, 100% of requests should have a TLS handshaking time of 0.
     */
    http_req_tls_handshaking: ['p(100)<0.001'],
    /**
     * Measures the time it takes to send a request to the server. Given the simplicity of the query,
     * the send time should be very short. Hence, 99% of requests should have a send time below 3ms.
     */
    http_req_sending: ['p(99)<3'],
    /**
     * Tracks the time spent waiting for the server to start sending a response after the request was sent.
     * The wait time should be relatively short given the simplicity of the query. Therefore,
     * 99% of requests should have a wait time below 30ms.
     */
    http_req_waiting: ['p(99)<30'],
    /**
     * Measures the time it takes to read the response from the server. Since the response payload (list of
     * country IDs) isn't large, this should be very quick. Hence, 99% of requests should have a receive
     * time below 5ms.
     */
    http_req_receiving: ['p(99)<5'],
    /**
     * Represents the total time for one VU iteration, including any setup and teardown scripts. Given
     * the simplicity of our test, 99% of iterations should complete within 1.5 second.
     */
    iteration_duration: ['p(99)<1500'],
  },
}

const trace = {
  http_req_duration: new Trend('http_req_duration', true),
  http_req_blocked: new Trend('http_req_blocked', true),
  http_req_connecting: new Trend('http_req_connecting', true),
  http_req_tls_handshaking: new Trend('http_req_tls_handshaking', true),
  http_req_sending: new Trend('http_req_sending', true),
  http_req_waiting: new Trend('http_req_waiting', true),
  http_req_receiving: new Trend('http_req_receiving', true),
  iteration_duration: new Trend('iteration_duration', true),
  my_iterations: new Trend('my_iterations', true),
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
  trace.http_req_duration.add(timings.duration)
  trace.http_req_blocked.add(timings.blocked)
  trace.http_req_connecting.add(timings.connecting)
  trace.http_req_tls_handshaking.add(timings.tls_handshaking)
  trace.http_req_sending.add(timings.sending)
  trace.http_req_waiting.add(timings.waiting)
  trace.http_req_receiving.add(timings.receiving)
  trace.iteration_duration.add(__VU)
  trace.my_iterations.add(__ITER)

  check(res, {
    'status was 200': (r) => r.status == 200,
    'status was not 500': (r) => r.status != 500,
    'transaction time OK': (r) => r.timings.duration < 200,
  })

  sleep(1)
}

export function handleSummary(data) {
  const data2 = Object.keys(data.metrics)
    .map((e) => {
      if (Object.keys(trace).includes(e)) {
        return { [e]: data.metrics[e] }
      } else {
        return null
      }
    })
    .filter(Boolean)

  // Customize the output to show only the essential metrics
  return {
    stdout: JSON.stringify(data2),
    './benches/actual/results.json': JSON.stringify(data2),
  }
}
