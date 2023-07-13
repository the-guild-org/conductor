import http from 'k6/http'

export let options = {
  stages: [
    { duration: '1m', target: 500 },
    // { duration: '2m', target: 500 },
    // { duration: '1m', target: 1000 },
    // { duration: '2m', target: 1000 },
    // { duration: '1m', target: 2000 },
    // { duration: '2m', target: 2000 },
    // { duration: '1m', target: 0 },
  ],
}

export default function () {
  let res = http.get('http://localhost:8001')
}
