#!/usr/bin/env node
const conductor = require('@graphql-conductor/lib')

function handleTerminationSignal(signal) {
  console.log(`Received ${signal}.`)
  conductor.shutdownServer()
}

process.on('SIGINT', handleTerminationSignal)
process.on('SIGTERM', handleTerminationSignal)

const configFilePath = process.argv[2] || './config.json'

conductor.executeConductor(configFilePath)?.catch((error) => {
  if (error) console.error('Error running conductor:', error)
})
