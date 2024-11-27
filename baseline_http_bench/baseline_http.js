import http from 'k6/http'
import { check, sleep } from 'k6'

export let options = {
  thresholds: {
    http_req_duration: ['p(95)<500'],
    http_req_failed: ['rate<0.01'],
  },
  stages: [
    { duration: '20s', target: 1000 },
    { duration: '1m', target: 1000 },
    { duration: '20s', target: 5000 },
    { duration: '2m', target: 5000 },
    { duration: '20s', target: 0 },
  ],
}

export default function () {
  const url = 'http://127.0.0.1:4000/baseline'
  const res = http.get(url)

  check(res, { 'status is 200': (r) => r.status === 200 })

  sleep(0.1)
}
