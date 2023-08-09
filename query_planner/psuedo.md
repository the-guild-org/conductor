Some Changes

User query: [
    FieldNode {
        field: "reviews",
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
                field: "rating",
                alias: None,
                arguments: [],
                children: [],
            },
            FieldNode {
                field: "location",
                alias: None,
                arguments: [],
                children: [
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
                ],
            },
        ],
    },
]

Supergraph Schema: {
    "REVIEWS": Subgraph {
        url: "\"http://localhost:4001/graphql\"",
        types: {
            "Query": TypeDetails {
                key: None,
                fields: {
                    "reviews": FieldDetails {
                        requires: None,
                        provides: None,
                        external: false,
                    },
                },
            },
        },
    },
    "LOCATIONS": Subgraph {
        url: "\"http://localhost:4002/graphql\"",
        types: {
            "Location": TypeDetails {
                key: Some(
                    "\"id\"",
                ),
                fields: {
                    "name": FieldDetails {
                        requires: None,
                        provides: None,
                        external: false,
                    },
                    "description": FieldDetails {
                        requires: None,
                        provides: None,
                        external: false,
                    },
                    "photo": FieldDetails {
                        requires: None,
                        provides: None,
                        external: false,
                    },
                },
            },
            "Query": TypeDetails {
                key: None,
                fields: {
                    "location": FieldDetails {
                        requires: None,
                        provides: None,
                        external: false,
                    },
                    "locations": FieldDetails {
                        requires: None,
                        provides: None,
                        external: false,
                    },
                },
            },
        },
    },
}