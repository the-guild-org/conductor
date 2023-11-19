/* eslint-disable */
import { createSchema } from "graphql-yoga";
import { faker } from "@faker-js/faker";
import { createServer } from "http";
import { createYoga } from "graphql-yoga";

function generateData() {
  const authors = [];
  for (let i = 0; i < 20; i++) {
    const books = [];

    for (let k = 0; k < 3; k++) {
      books.push({
        id: faker.string.uuid(),
        name: faker.internet.domainName(),
        numPages: faker.number.int({
          min: 1,
          max: 1000,
        }),
      });
    }

    authors.push({
      id: faker.string.uuid(),
      name: faker.person.fullName(),
      company: faker.company.buzzPhrase(),
      books,
    });
  }

  return authors;
}

const data = generateData();

export const schema = createSchema({
  typeDefs: /* GraphQL */ `
    type Author {
      id: ID!
      name: String!
      company: String!
      books: [Book!]!
    }
    type Book {
      id: ID!
      name: String!
      numPages: Int!
    }
    type Query {
      authors: [Author!]!
    }
  `,
  resolvers: {
    Author: {},
    Query: {
      authors: () => data,
    },
  },
});

const yoga = createYoga({
  schema,
  logging: false,
  multipart: false,
});

const server = createServer((req, res) => {
  if (req.url === "/_health") {
    res.writeHead(200);
    return res.end();
  } else {
    return yoga(req, res);
  }
});

server.listen(process.env.PORT ? parseInt(process.env.PORT) : 4000, () => {
  console.log("ready");
});
