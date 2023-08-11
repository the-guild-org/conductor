import { parse } from 'graphql'
import { buildSubgraphSchema } from '@apollo/subgraph'
import { createYoga } from 'graphql-yoga'
import { createServer } from 'http'

const typeDefs = parse(/* GraphQL */ `
  type Query {
    locations: [Location!]!
    location(id: ID!): Location
  }

  type Location @key(fields: "id") {
    id: ID!
    name: String! 
    description: String! 
    photo: String!
    isCool: IsCool!
  }

  type IsCool {
    really: IsCoolReally!
  }

  type IsCoolReally {
    yes: Boolean!
  }
`)

const location1 = {
  id: 'portugal',
  name: 'Portugal - European Union',
  description: 'The best country in the world',
  photo: 'https://....',
  isCool: { really: { yes: true } }
}

const location2 = {
  id: 'usa',
  name: 'USA - United States Of America',
  description: 'The worst country in the world',
  photo: 'https://....',
  isCool: { really: { yes: false } }
}

const locations = [location1, location2]

const resolvers = {
  Query: {
    location(_: any, { id }: { id: string }) {
      return locations.find((e) => e.id === id)
    },
    locations() {
      return locations
    }
  },
  // Location: {
  //   __resolveReference(location: any) {
  //     return locations.find((loc: any) => loc.id === location.id)
  //   }
  // }
}

const yoga = createYoga({
  schema: buildSubgraphSchema([{ typeDefs, resolvers }])
})

const server = createServer(yoga)

server.listen(4002, () => {
  console.log(`🚀 Server ready at http://localhost:4002`)
})
