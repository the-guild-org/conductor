import http from 'k6/http'
import { Trend } from 'k6/metrics'

export let options = {
  stages: [
    { duration: '1m', target: 500 / 2 },
    { duration: '2m', target: 500 },
    // { duration: '1m', target: 1000 },
    // { duration: '2m', target: 1000 },
    // { duration: '1m', target: 2000 },
    // { duration: '2m', target: 2000 },
    { duration: '1m', target: 0 },
  ],
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
  let res = http.get('http://localhost:8001')

  const timings = res.timings || {}
  timings.duration && trace.http_req_duration.add(timings.duration)
  timings.blocked && trace.http_req_blocked.add(timings.blocked)
  timings.connecting && trace.http_req_connecting.add(timings.connecting)
  timings.tlsHandshaking &&
    trace.http_req_tls_handshaking.add(timings.tlsHandshaking)
  timings.sending && trace.http_req_sending.add(timings.sending)
  timings.waiting && trace.http_req_waiting.add(timings.waiting)
  timings.receiving && trace.http_req_receiving.add(timings.receiving)

  trace.iteration_duration.add(__VU)
  trace.my_iterations.add(__ITER)
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

  return {
    stdout: JSON.stringify(data.metrics),
    './benches/baseline/results.json': JSON.stringify(data.metrics),
  }
}
