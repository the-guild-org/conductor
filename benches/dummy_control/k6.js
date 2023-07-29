import http from 'k6/http'
import { Trend } from 'k6/metrics'

const VUS = 1000 // 1000 virtual users

export let options = {
  stages: [
    { duration: '2m', target: VUS }, // Warm up stage
    { duration: '1m', target: VUS * 3 }, // Sustained high load
    // { duration: '2m', target: VUS * 2 }, // Ramp up to very high load
    // { duration: '5m', target: VUS * 2 }, // Sustained very high load
    // { duration: '2m', target: VUS * 3 }, // Ramp up to extreme load
    // { duration: '5m', target: VUS * 3 }, // Sustained extreme load
    { duration: '2m', target: 0 }, // Scale down
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
    './benches/dummy_control/results.json': JSON.stringify(data2),
  }
}
