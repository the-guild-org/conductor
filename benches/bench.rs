// How to run the bench?
// 1. Use one of the below parsers
// 2. Run `cargo bench -- --save-baseline testing-speed`
// 3. Remember which parser you saved as the baseline
// 4. cargo bench -- --baseline testing-speed

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use graphql_parser::{parse_query, parse_schema};
// Switch between `graphql_parser` and `async_graphql_parser`
// use async_graphql_parser::{parse_query, parse_schema};

const SAMPLE_SIZE: usize = 100;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Parse User Schema", |b| {
        b.iter(|| {
            let query = parse_query::<String>(
                r#"
              fragment User on User {
    id
    username
    name
  }

  fragment Review on Review {
    id
    body
  }

  fragment Product on Product {
    inStock
    name
    price
    shippingEstimate
    upc
    weight
  }

  query TestQuery {
    users {
      ...User
      reviews {
        ...Review
        product {
          ...Product
          reviews {
            ...Review
            author {
              ...User
              reviews {
                ...Review
                product {
                  ...Product
                }
              }
            }
          }
        }
      }
    }
    topProducts {
      ...Product
      reviews {
        ...Review
        author {
          ...User
          reviews {
            ...Review
            product {
              ...Product
            }
          }
        }
      }
    }
  }
            "#,
            );

            black_box(query)
        })
    });
    c.bench_function("Parse Supergraph Schema", |b| {
        b.iter(|| {
            let supergraph = parse_schema::<String>(
                r#"
            schema
  @core(feature: "https://specs.apollo.dev/core/v0.2")
  @core(feature: "https://specs.apollo.dev/join/v0.1", for: EXECUTION) {
  query: Query
}

directive @core(as: String, feature: String!, for: core__Purpose) repeatable on SCHEMA

directive @join__field(
  graph: join__Graph
  provides: join__FieldSet
  requires: join__FieldSet
) on FIELD_DEFINITION

directive @join__graph(name: String!, url: String!) on ENUM_VALUE

directive @join__owner(graph: join__Graph!) on INTERFACE | OBJECT

directive @join__type(graph: join__Graph!, key: join__FieldSet) repeatable on INTERFACE | OBJECT

type Product
  @join__owner(graph: SERVICE1)
  @join__type(graph: SERVICE1, key: "upc")
  @join__type(graph: SERVICE2, key: "upc")
  @join__type(graph: SERVICE3, key: "upc") {
  inStock: Boolean @join__field(graph: SERVICE3)
  name: String @join__field(graph: SERVICE1)
  price: Int @join__field(graph: SERVICE1)
  reviews: [Review] @join__field(graph: SERVICE2)
  shippingEstimate: Int @join__field(graph: SERVICE3, requires: "price weight")
  upc: String! @join__field(graph: SERVICE1)
  weight: Int @join__field(graph: SERVICE1)
}

type Query {
  me: User @join__field(graph: SERVICE0)
  topProducts(first: Int): [Product] @join__field(graph: SERVICE1)
  users: [User] @join__field(graph: SERVICE0)
}

type Review @join__owner(graph: SERVICE2) @join__type(graph: SERVICE2, key: "id") {
  author: User @join__field(graph: SERVICE2, provides: "username")
  body: String @join__field(graph: SERVICE2)
  id: ID! @join__field(graph: SERVICE2)
  product: Product @join__field(graph: SERVICE2)
}

type User
  @join__owner(graph: SERVICE0)
  @join__type(graph: SERVICE0, key: "id")
  @join__type(graph: SERVICE2, key: "id") {
  birthDate: String @join__field(graph: SERVICE0)
  id: ID! @join__field(graph: SERVICE0)
  name: String @join__field(graph: SERVICE0)
  numberOfReviews: Int @join__field(graph: SERVICE2)
  reviews: [Review] @join__field(graph: SERVICE2)
  username: String @join__field(graph: SERVICE0)
}

enum core__Purpose {
  """
  `EXECUTION` features provide metadata necessary to for operation execution.
  """
  EXECUTION

  """
  `SECURITY` features provide metadata necessary to securely resolve fields.
  """
  SECURITY
}

scalar join__FieldSet

enum join__Graph {
  SERVICE0 @join__graph(name: "service0", url: "http://www.service-0.com")
  SERVICE1 @join__graph(name: "service1", url: "http://www.service-1.com")
  SERVICE2 @join__graph(name: "service2", url: "http://www.service-2.com")
  SERVICE3 @join__graph(name: "service3", url: "http://www.service-3.com")
}
"#,
            );

            black_box(supergraph)
        })
    });
}

fn configure_benchmark() -> Criterion {
    Criterion::default().sample_size(SAMPLE_SIZE)
}

criterion_group! {
    name = benches;
    config = configure_benchmark();
    targets = criterion_benchmark
}
criterion_main!(benches);
