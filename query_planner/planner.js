const userQuery = [
  {
    field: 'locations',
    alias: null,
    arguments: [],
    children: [
      {
        field: 'id',
        alias: null,
      },
      {
        field: 'name',
        alias: null,
      },
      {
        field: 'description',
        alias: null,
      },
      {
        field: 'reviews',
        alias: null,
        children: [
          {
            field: 'comment',
            alias: null,
          },
          {
            field: 'rating',
            alias: null,
          },
        ],
      },
    ],
  },
]

const supergraphSchema = {
  services: {
    LOCATIONS: {
      url: 'http://localhost:4002/graphql',
    },
    REVIEWS: {
      url: 'http://localhost:4001/graphql',
    },
  },
  types: {
    Query: {
      service: 'ROOT',
      fields: {
        locations: {
          service: 'LOCATIONS',
          type: 'Location',
        },
        location: {
          service: 'LOCATIONS',
          type: 'Location',
        },
      },
    },
    Location: {
      service: 'LOCATIONS',
      key: 'id',
      fields: {
        id: {
          service: 'LOCATIONS',
        },
        name: {
          service: 'LOCATIONS',
        },
        description: {
          service: 'LOCATIONS',
        },
        photo: {
          service: 'LOCATIONS',
        },
        reviews: {
          service: 'REVIEWS',
          type: 'Review',
        },
      },
    },
    Review: {
      service: 'REVIEWS',
      key: 'id',
      fields: {
        id: {
          service: 'REVIEWS',
        },
        comment: {
          service: 'REVIEWS',
        },
        rating: {
          service: 'REVIEWS',
        },
        wowLocationID: {
          service: 'REVIEWS',
        },
      },
    },
  },
}

function createQueryPlan(userQuery, supergraphSchema) {
  let queryPlan = []

  function traverse(
    query,
    schema,
    parentType = { type: 'Query', service: null }
  ) {
    let graph

    if (query.field) {
      if (schema.types[parentType.type].fields.hasOwnProperty(query.field)) {
        graph = schema.types[parentType.type].fields[query.field].service
      } else {
        throw new Error(
          `The field "${query.field}" was not found in the schema types.`
        )
      }

      if (parentType.service && graph !== parentType.service) {
        const sequenceStep = {
          step: 'SEQUENCE',
          operations: [],
        }

        sequenceStep.operations.push({
          serviceName: parentType.service,
          serviceURL: schema.services[parentType.service].url,
          queryFields: [query],
        })

        sequenceStep.operations.push({
          serviceName: graph,
          serviceURL: schema.services[graph].url,
          queryFields: [
            {
              field: '_entities',
              alias: null,
              arguments: [
                {
                  name: 'representations',
                  value: [
                    {
                      __typename: parentType.type,
                      id: `$${parentType.type.toLowerCase()}.id`,
                    },
                  ],
                },
              ],
              children: [query],
            },
          ],
        })

        queryPlan.push(sequenceStep)
      } else {
        const existingOperationIndex = queryPlan.findIndex(
          (step) => step.operations && step.operations[0].serviceName === graph
        )

        if (existingOperationIndex === -1) {
          queryPlan.push({
            serviceName: graph,
            serviceURL: schema.services[graph].url,
            queryFields: [query],
          })
        } else {
          queryPlan[existingOperationIndex].operations[0].queryFields.push(
            query
          )
        }
      }
    }

    if (query.children) {
      for (let child of query.children) {
        traverse(
          child,
          schema,
          schema.types[parentType.type].fields[query.field]
        )
      }
    }
  }

  userQuery.forEach((queryItem) => traverse(queryItem, supergraphSchema))

  return queryPlan
}

function getGraphContainingField(supergraphSchema, field) {
  for (let graph in supergraphSchema) {
    for (let type in supergraphSchema[graph].types) {
      let fields = supergraphSchema[graph].types[type].fields
      if (Array.isArray(fields) && fields.includes(field)) {
        return graph
      }
    }
  }
  return null
}

let queryPlan = createQueryPlan(userQuery, supergraphSchema)
console.log(JSON.stringify(queryPlan, null, 2))
executeQueryPlan(queryPlan).then((data) => console.log(data))

async function executeQueryPlan(queryPlan) {
  let data = {}
  let cache = {}

  for (let step of queryPlan) {
    if (step.step === 'SEQUENCE') {
      for (let operation of step.operations) {
        let response = await fetch(operation.serviceURL, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            query: operation.queryFields
              .map(
                (field) =>
                  `${field.field} { ${(field.children || [])
                    .map((child) => child.field)
                    .join(', ')} }`
              )
              .join(', '),
          }),
        })
        let responseData = await response.json()
        cache = { ...cache, ...responseData.data }
      }
    } else {
      let response = await fetch(step.serviceURL, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          query: step.queryFields
            .map(
              (field) =>
                `${field.field} { ${(field.children || [])
                  .map((child) => child.field)
                  .join(', ')} }`
            )
            .join(', '),
        }),
      })
      let responseData = await response.json()
      data = { ...data, ...responseData.data }
    }
  }

  return data
}
