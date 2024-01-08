const fs = require('fs')
const axios = require('axios')

const markdownContent = fs.readFileSync('panic-audit.md', 'utf8')

const data = markdownContent

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

async function githubComment(markdown, options) {
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

  const existingComment = await getExistingComment(prNumber)

  const body = markdown

  if (existingComment) {
    console.log('Updating existing PR comment...')
    await updateComment(existingComment.id, body)
  } else {
    console.log('Creating a new PR comment...')
    await createComment(prNumber, body)
  }

  async function getPullRequestNumber() {
    try {
      const response = await axios.get(
        `https://api.github.com/repos/${org}/${repo}/commits/${commit}/pulls`,
        {
          headers: {
            accept: 'application/vnd.github.groot-preview+json',
            Authorization: `Bearer ${token}`,
          },
        }
      )

      const pullRequests = response.data

      if (pullRequests && pullRequests.length) {
        return pullRequests[0].number
      }

      return null
    } catch (error) {
      console.error('Error fetching Pull Request number:', error)
      return null
    }
  }

  async function getExistingComment(id) {
    try {
      const response = await axios.get(
        `https://api.github.com/repos/${org}/${repo}/issues/${id}/comments`,
        {
          headers: {
            accept: 'application/vnd.github.v3+json',
            Authorization: `Bearer ${token}`,
          },
        }
      )

      const comments = response.data

      if (comments && comments.length) {
        return matchComment(comments)
      }

      return null
    } catch (error) {
      console.error('Error fetching existing comment:', error)
      return null
    }
  }

  async function updateComment(id, body) {
    try {
      await axios.patch(
        `https://api.github.com/repos/${org}/${repo}/issues/comments/${id}`,
        {
          body,
        },
        {
          headers: {
            accept: 'application/vnd.github.v3+json',
            Authorization: `Bearer ${token}`,
          },
        }
      )
    } catch (error) {
      console.error('Error updating comment:', error)
    }
  }

  async function createComment(id, body) {
    try {
      await axios.post(
        `https://api.github.com/repos/${org}/${repo}/issues/${id}/comments`,
        {
          body,
        },
        {
          headers: {
            accept: 'application/vnd.github.v3+json',
            Authorization: `Bearer ${token}`,
          },
        }
      )
    } catch (error) {
      console.error('Error creating comment:', error)
    }
  }
}

function matchComment(comments) {
  return comments.find(({ body }) => {
    return body.includes('http_req_waiting')
  })
}
