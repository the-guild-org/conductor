const { createServer } = require('node:http')
const { createYoga, createSchema } = require('graphql-yoga')

const schema = createSchema({
  typeDefs: /* GraphQL */ `
    type Query {
      country(code: ID!): Country
    }

    type Country {
      name: String
      code: ID!
    }
  `,
  resolvers: {
    Query: {
      country: () => ({
        code: 'EG',
        name: 'Egypt',
      }),
    },
  },
})

const yoga = createYoga({ schema })
const server = createServer(yoga)

// Start the server and you're done!
server.listen(4000, () => {
  console.info('Server is running on http://localhost:4000/graphql')
})
