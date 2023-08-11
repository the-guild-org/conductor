import { parse } from 'graphql'
import { buildSubgraphSchema } from '@apollo/subgraph'
import { createYoga } from 'graphql-yoga'
import { createServer } from 'http'

const typeDefs = parse(/* GraphQL */ `
  type Review @key(fields: "id") {
    id: ID!
    comment: String
    rating: Int
    wowLocationID: ID!
  }

  type Location @key(fields: "id") {
    id: ID! @external
    reviews: [Review!]!
  }
`)

const review1 = { id: '1', comment: "Wow! Portugal is amazing!", rating: 9, wowLocationID: 'portugal' }
const review2 = { id: '2', comment: "USA could be better...", rating: 6, wowLocationID: 'usa' }

const reviews = [review1, review2]

const resolvers = {
  Location: {
    reviews(location: any) {
      console.log(location)
      return reviews.filter(review => review.wowLocationID === location.id)
    }
  },
  // Location: {
  //   reviews() {
  //     return reviews
  //   },
  // },
  // Review: {
  //   location(review: any) {
  //     return {
  //       __typename: 'Location',
  //       id: reviews.find((e) => review.location.id === e.location.id)?.location.id
  //     }
  //   }
  // }
}


const yoga = createYoga({
  schema: buildSubgraphSchema([{ typeDefs, resolvers }])
})

const server = createServer(yoga)

server.listen(4001, () => {
  console.log(`🚀 Server ready at http://localhost:4001`)
})
