function generateHugeQuery() {
  let query = 'query HUGE_QUERY {\n'

  for (let i = 0; i < 100; i++) {
    query += `
    locations${i} {
        name${i}
        description${i}
        reviews${i} {
            comment${i}
            rating${i}
        }
        isCool${i} {
            really${i} {
                yes${i}
            }
        }
    }

    location${i}(id: "country${i}") {
        photo${i}
    }
        `
  }

  query += '}'
  return query
}

const hugeQuery = generateHugeQuery()
require('fs').writeFileSync('huge-query.graphql', hugeQuery)
