import http from 'k6/http'

export let options = {
  stages: [
    { duration: '1m', target: 500 }, // ramp up to 500 VUs (virtual users)
    // { duration: '2m', target: 500 }, // stay at 500 VUs
    // { duration: '1m', target: 1000 }, // ramp up to 1000 VUs
    // { duration: '2m', target: 1000 }, // stay at 1000 VUs
    // { duration: '1m', target: 2000 }, // ramp up to 2000 VUs
    // { duration: '2m', target: 2000 }, // stay at 2000 VUs
    // { duration: '1m', target: 0 }, // scale down. Recovery stage.
  ],
}

export default function () {
  let query = `
    query GetCountryCode($code: ID!) {
        country(code: $code) {
            name
        }
    }`

  let variables = {
    code: 'EG',
  }

  let body = JSON.stringify({
    query: query,
    variables: variables,
    operationName: 'GetCountryCode',
  })

  let headers = { 'Content-Type': 'application/json' }

  let _res = http.post('http://localhost:4000/graphql', body, {
    headers,
  })
  // Here we don't sleep after each request, to generate as high load as possible
}
