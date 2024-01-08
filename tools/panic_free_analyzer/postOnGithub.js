import fs from 'fs'
import { githubComment } from 'https://raw.githubusercontent.com/dotansimha/k6-github-pr-comment/master/lib.js'

const markdownContent = fs.readFileSync('panic-audit.md', 'utf8')

const data = {
  summary: markdownContent,
}

if (process.env.GITHUB_TOKEN) {
  githubComment(data, {
    token: process.env.GITHUB_TOKEN,
    commit: process.env.GITHUB_SHA,
    pr: process.env.GITHUB_PR,
    org: 'the-guild-org',
    repo: 'conductor-t2',
    renderTitle() {
      return ''
    },
    renderMessage() {
      return ''
    },
  })
} else {
  console.log('GITHUB_TOKEN is not set. Unable to post comment.')
}
