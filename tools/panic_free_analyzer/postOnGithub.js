const fs = require('fs')
const http = require('https')

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

// simplified version of: https://raw.githubusercontent.com/dotansimha/k6-github-pr-comment/master/lib.js

function githubComment(markdown, options) {
  if (!options.commit) {
    return
  }

  const token = options.token
  const commit = options.commit
  const org = options.org
  const repo = options.repo

  const prNumber = options.pr || getPullRequestNumber()

  if (!prNumber) {
    console.log('Not a Pull Request. Skipping comment creation...')
    return
  }

  const existingComment = getExistingComment(prNumber)

  const body = markdown

  if (existingComment) {
    console.log('Updating existing PR comment...')
    updateComment(existingComment.id, body)
  } else {
    console.log('Creating a new PR comment...')
    createComment(prNumber, body)
  }

  function getPullRequestNumber() {
    const res = http.get(
      `https://api.github.com/repos/${org}/${repo}/commits/${commit}/pulls`,
      {
        headers: {
          accept: 'application/vnd.github.groot-preview+json',
          authorization: `Bearer ${token}`,
        },
      }
    )

    const pullRequests = res.json()

    if (pullRequests && pullRequests.length) {
      return pullRequests[0].number
    }

    return null
  }

  function getExistingComment(id) {
    const res = http.get(
      `https://api.github.com/repos/${org}/${repo}/issues/${id}/comments`,
      {
        headers: {
          accept: 'application/vnd.github.v3+json',
          authorization: `Bearer ${token}`,
        },
      }
    )

    const comments = res.json()

    if (comments && comments.length) {
      return matchComment(comments)
    }

    return null
  }

  function updateComment(id, body) {
    const res = http.patch(
      `https://api.github.com/repos/${org}/${repo}/issues/comments/${id}`,
      JSON.stringify({
        body,
      }),
      {
        headers: {
          accept: 'application/vnd.github.v3+json',
          authorization: `Bearer ${token}`,
        },
      }
    )

    assert2XX(res, 'Failed to update the comment')
  }

  function createComment(id, body) {
    const res = http.post(
      `https://api.github.com/repos/${org}/${repo}/issues/${id}/comments`,
      JSON.stringify({
        body,
      }),
      {
        headers: {
          accept: 'application/vnd.github.v3+json',
          authorization: `Bearer ${token}`,
        },
      }
    )

    assert2XX(res, 'Failed to create a comment')
  }
}

function assert2XX(res, message) {
  if (res.status === 200 || res.status === 201) {
    return
  }

  if (res.status < 200 && res.status >= 300) {
    console.error(message, res.status, res.error, res.error_code)
  } else {
    console.warn(message, res.status, res.error, res.error_code)
  }
}

function matchComment(comments) {
  return comments.find(({ body }) => {
    return body.includes('http_req_waiting')
  })
}
