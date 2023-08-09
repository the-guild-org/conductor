Supergraph Supergraph {
    types: {
        "Location": GraphQLType {
            key_fields: [
                "id",
            ],
            fields: {
                "id": GraphQLField {
                    field_type: "ID!",
                    source: "LOCATIONS",
                    requires: None,
                    provides: None,
                    external: true,
                },
                "photo": GraphQLField {
                    field_type: "String!",
                    source: "LOCATIONS",
                    requires: None,
                    provides: None,
                    external: false,
                },
                "description": GraphQLField {
                    field_type: "String!",
                    source: "LOCATIONS",
                    requires: None,
                    provides: None,
                    external: false,
                },
                "name": GraphQLField {
                    field_type: "String!",
                    source: "LOCATIONS",
                    requires: None,
                    provides: None,
                    external: false,
                },
                "isCool": GraphQLField {
                    field_type: "IsCool!",
                    source: "LOCATIONS",
                    requires: None,
                    provides: None,
                    external: false,
                },
                "reviews": GraphQLField {
                    field_type: "[Review!]!",
                    source: "REVIEWS",
                    requires: None,
                    provides: None,
                    external: false,
                },
            },
        },
        "Query": GraphQLType {
            key_fields: [],
            fields: {
                "location": GraphQLField {
                    field_type: "Location",
                    source: "LOCATIONS",
                    requires: None,
                    provides: None,
                    external: false,
                },
                "locations": GraphQLField {
                    field_type: "[Location!]!",
                    source: "LOCATIONS",
                    requires: None,
                    provides: None,
                    external: false,
                },
            },
        },
        "Review": GraphQLType {
            key_fields: [
                "id",
            ],
            fields: {
                "rating": GraphQLField {
                    field_type: "Int",
                    source: "REVIEWS",
                    requires: None,
                    provides: None,
                    external: false,
                },
                "comment": GraphQLField {
                    field_type: "String",
                    source: "REVIEWS",
                    requires: None,
                    provides: None,
                    external: false,
                },
                "wowLocationID": GraphQLField {
                    field_type: "ID!",
                    source: "REVIEWS",
                    requires: None,
                    provides: None,
                    external: false,
                },
                "id": GraphQLField {
                    field_type: "ID!",
                    source: "REVIEWS",
                    requires: None,
                    provides: None,
                    external: false,
                },
            },
        },
        "IsCool": GraphQLType {
            key_fields: [],
            fields: {
                "really": GraphQLField {
                    field_type: "Boolean!",
                    source: "LOCATIONS",
                    requires: None,
                    provides: None,
                    external: false,
                },
            },
        },
    },
    services: {
        "REVIEWS": "http://localhost:4001/graphql",
        "LOCATIONS": "http://localhost:4002/graphql",
    },
}
User query: UserQuery {
    operation_type: Query,
    operation_name: None,
    arguments: [],
    fields: [
        FieldNode {
            field: "locations",
            alias: None,
            arguments: [],
            children: [
                FieldNode {
                    field: "id",
                    alias: None,
                    arguments: [],
                    children: [],
                },
                FieldNode {
                    field: "name",
                    alias: None,
                    arguments: [],
                    children: [],
                },
                FieldNode {
                    field: "description",
                    alias: None,
                    arguments: [],
                    children: [],
                },
                FieldNode {
                    field: "reviews",
                    alias: None,
                    arguments: [],
                    children: [
                        FieldNode {
                            field: "comment",
                            alias: None,
                            arguments: [],
                            children: [],
                        },
                        FieldNode {
                            field: "rating",
                            alias: None,
                            arguments: [],
                            children: [],
                        },
                    ],
                },
                FieldNode {
                    field: "isCool",
                    alias: None,
                    arguments: [],
                    children: [
                        FieldNode {
                            field: "really",
                            alias: None,
                            arguments: [],
                            children: [],
                        },
                    ],
                },
            ],
        },
        FieldNode {
            field: "location",
            alias: None,
            arguments: [
                (
                    "id",
                    String(
                        "portugal",
                    ),
                ),
            ],
            children: [
                FieldNode {
                    field: "photo",
                    alias: None,
                    arguments: [],
                    children: [],
                },
            ],
        },
    ],
}