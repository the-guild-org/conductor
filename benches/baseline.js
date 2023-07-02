const http = require('http')

http
  .createServer((req, res) => {
    res.writeHead(200, { 'Content-Type': 'text/plain' })
    res.end('Hello, World!')
  })
  .listen(8001, () => {
    console.log('Baseline server running on http://localhost:8001')
  })
