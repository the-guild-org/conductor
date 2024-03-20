use crate::graphql::ParsedGraphQLSchema;
use graphql_parser::schema::{
  Definition, EnumType, EnumValue, Field, InputObjectType, InputValue, InterfaceType, ObjectType,
  ScalarType, SchemaDefinition, Type, TypeDefinition, UnionType,
};
pub use graphql_tools::introspection::IntrospectionQuery as IntrospectionQueryResponse;
use graphql_tools::introspection::{
  IntrospectionInputTypeRef, IntrospectionOutputTypeRef, IntrospectionType,
};

fn serde_value_to_sdl_value(
  value: serde_json::Value,
) -> graphql_parser::schema::Value<'static, String> {
  match value {
    serde_json::Value::Null => graphql_parser::schema::Value::Null,
    serde_json::Value::Bool(v) => graphql_parser::schema::Value::Boolean(v),
    serde_json::Value::Number(v) => {
      if v.is_i64() {
        graphql_parser::schema::Value::Int((v.as_i64().unwrap() as i32).into())
      } else {
        graphql_parser::schema::Value::Float(v.as_f64().unwrap())
      }
    }
    serde_json::Value::String(v) => graphql_parser::schema::Value::String(v),
    serde_json::Value::Array(v) => {
      graphql_parser::schema::Value::List(v.into_iter().map(serde_value_to_sdl_value).collect())
    }
    serde_json::Value::Object(v) => graphql_parser::schema::Value::Object(
      v.into_iter()
        .map(|(k, v)| (k, serde_value_to_sdl_value(v)))
        .collect(),
    ),
  }
}

fn introspection_input_typeref_to_sdl_type(
  type_ref: &IntrospectionInputTypeRef,
) -> Type<'static, String> {
  match type_ref {
    IntrospectionInputTypeRef::SCALAR(s) => Type::NamedType(s.name.clone()),
    IntrospectionInputTypeRef::ENUM(e) => Type::NamedType(e.name.clone()),
    IntrospectionInputTypeRef::INPUT_OBJECT(i) => Type::NamedType(i.name.clone()),
    IntrospectionInputTypeRef::NON_NULL { of_type } => Type::NonNullType(Box::new(
      introspection_input_typeref_to_sdl_type(of_type.as_ref().unwrap()),
    )),
    IntrospectionInputTypeRef::LIST { of_type } => Type::ListType(Box::new(
      introspection_input_typeref_to_sdl_type(of_type.as_ref().unwrap()),
    )),
  }
}

fn introspection_output_typeref_to_sdl_type(
  type_ref: &IntrospectionOutputTypeRef,
) -> Type<'static, String> {
  match type_ref {
    IntrospectionOutputTypeRef::INTERFACE(s) => Type::NamedType(s.name.clone()),
    IntrospectionOutputTypeRef::OBJECT(s) => Type::NamedType(s.name.clone()),
    IntrospectionOutputTypeRef::UNION(s) => Type::NamedType(s.name.clone()),
    IntrospectionOutputTypeRef::SCALAR(s) => Type::NamedType(s.name.clone()),
    IntrospectionOutputTypeRef::ENUM(e) => Type::NamedType(e.name.clone()),
    IntrospectionOutputTypeRef::INPUT_OBJECT(i) => Type::NamedType(i.name.clone()),
    IntrospectionOutputTypeRef::NON_NULL { of_type } => Type::NonNullType(Box::new(
      introspection_output_typeref_to_sdl_type(of_type.as_ref().unwrap()),
    )),
    IntrospectionOutputTypeRef::LIST { of_type } => Type::ListType(Box::new(
      introspection_output_typeref_to_sdl_type(of_type.as_ref().unwrap()),
    )),
  }
}

pub fn introspection_to_sdl(introspection: IntrospectionQueryResponse) -> ParsedGraphQLSchema {
  let schema_type = introspection.__schema;
  let schema_definition = Definition::SchemaDefinition(SchemaDefinition {
    directives: vec![],
    position: Default::default(),
    mutation: schema_type.mutation_type.map(|v| v.name),
    query: Some(schema_type.query_type.name),
    subscription: schema_type.subscription_type.map(|v| v.name),
  });
  let mut result = ParsedGraphQLSchema {
    definitions: vec![schema_definition],
  };

  for introspection_directive in schema_type.directives {
    result.definitions.push(Definition::DirectiveDefinition(
      graphql_parser::schema::DirectiveDefinition {
        description: introspection_directive.description,
        name: introspection_directive.name,
        position: Default::default(),
        arguments: introspection_directive
          .args
          .into_iter()
          .map(|introspection_input_value| InputValue {
            default_value: introspection_input_value
              .default_value
              .map(serde_value_to_sdl_value),
            value_type: introspection_input_typeref_to_sdl_type(
              &introspection_input_value.type_ref.unwrap(),
            ),
            description: introspection_input_value.description,
            directives: vec![],
            name: introspection_input_value.name,
            position: Default::default(),
          })
          .collect(),
        repeatable: introspection_directive.is_repeatable.is_some_and(|v| v),
        locations: introspection_directive
          .locations
          .into_iter()
          .map(|v| match v {
            graphql_tools::introspection::DirectiveLocation::ARGUMENT_DEFINITION => {
              graphql_parser::schema::DirectiveLocation::ArgumentDefinition
            }
            graphql_tools::introspection::DirectiveLocation::VARIABLE_DEFINITION => {
              graphql_parser::schema::DirectiveLocation::VariableDefinition
            }
            graphql_tools::introspection::DirectiveLocation::ENUM => {
              graphql_parser::schema::DirectiveLocation::Enum
            }
            graphql_tools::introspection::DirectiveLocation::UNION => {
              graphql_parser::schema::DirectiveLocation::Union
            }
            graphql_tools::introspection::DirectiveLocation::ENUM_VALUE => {
              graphql_parser::schema::DirectiveLocation::EnumValue
            }
            graphql_tools::introspection::DirectiveLocation::FIELD => {
              graphql_parser::schema::DirectiveLocation::Field
            }
            graphql_tools::introspection::DirectiveLocation::FIELD_DEFINITION => {
              graphql_parser::schema::DirectiveLocation::FieldDefinition
            }
            graphql_tools::introspection::DirectiveLocation::FRAGMENT_DEFINITION => {
              graphql_parser::schema::DirectiveLocation::FragmentDefinition
            }
            graphql_tools::introspection::DirectiveLocation::FRAGMENT_SPREAD => {
              graphql_parser::schema::DirectiveLocation::FragmentSpread
            }
            graphql_tools::introspection::DirectiveLocation::INLINE_FRAGMENT => {
              graphql_parser::schema::DirectiveLocation::InlineFragment
            }
            graphql_tools::introspection::DirectiveLocation::INPUT_FIELD_DEFINITION => {
              graphql_parser::schema::DirectiveLocation::InputFieldDefinition
            }
            graphql_tools::introspection::DirectiveLocation::INPUT_OBJECT => {
              graphql_parser::schema::DirectiveLocation::InputObject
            }
            graphql_tools::introspection::DirectiveLocation::INTERFACE => {
              graphql_parser::schema::DirectiveLocation::Interface
            }
            graphql_tools::introspection::DirectiveLocation::MUTATION => {
              graphql_parser::schema::DirectiveLocation::Mutation
            }
            graphql_tools::introspection::DirectiveLocation::OBJECT => {
              graphql_parser::schema::DirectiveLocation::Object
            }
            graphql_tools::introspection::DirectiveLocation::QUERY => {
              graphql_parser::schema::DirectiveLocation::Query
            }
            graphql_tools::introspection::DirectiveLocation::SCALAR => {
              graphql_parser::schema::DirectiveLocation::Scalar
            }
            graphql_tools::introspection::DirectiveLocation::SCHEMA => {
              graphql_parser::schema::DirectiveLocation::Schema
            }
            graphql_tools::introspection::DirectiveLocation::SUBSCRIPTION => {
              graphql_parser::schema::DirectiveLocation::Subscription
            }
          })
          .collect(),
      },
    ));
  }

  for introspection_type in schema_type.types {
    let schema_type = match introspection_type {
      IntrospectionType::INTERFACE(introspection_interface) => {
        TypeDefinition::Interface(InterfaceType {
          description: introspection_interface.description,
          directives: vec![],
          name: introspection_interface.name,
          position: Default::default(),
          implements_interfaces: introspection_interface
            .possible_types
            .into_iter()
            .map(|v| v.name)
            .collect(),
          fields: introspection_interface
            .fields
            .into_iter()
            .map(|introspection_field| Field {
              description: introspection_field.description,
              directives: vec![],
              name: introspection_field.name,
              position: Default::default(),
              arguments: introspection_field
                .args
                .into_iter()
                .map(|introspection_input_value| InputValue {
                  default_value: introspection_input_value
                    .default_value
                    .map(serde_value_to_sdl_value),
                  value_type: introspection_input_typeref_to_sdl_type(
                    &introspection_input_value.type_ref.unwrap(),
                  ),
                  description: introspection_input_value.description,
                  directives: vec![],
                  name: introspection_input_value.name,
                  position: Default::default(),
                })
                .collect(),
              field_type: introspection_output_typeref_to_sdl_type(&introspection_field.type_ref),
            })
            .collect(),
        })
      }
      IntrospectionType::OBJECT(introspection_object_type) => TypeDefinition::Object(ObjectType {
        description: introspection_object_type.description,
        directives: vec![],
        name: introspection_object_type.name,
        position: Default::default(),
        implements_interfaces: introspection_object_type
          .interfaces
          .into_iter()
          .map(|v| v.name)
          .collect(),
        fields: introspection_object_type
          .fields
          .into_iter()
          .map(|introspection_field| Field {
            description: introspection_field.description,
            directives: vec![],
            name: introspection_field.name,
            position: Default::default(),
            arguments: introspection_field
              .args
              .into_iter()
              .map(|introspection_input_value| InputValue {
                default_value: introspection_input_value
                  .default_value
                  .map(serde_value_to_sdl_value),
                value_type: introspection_input_typeref_to_sdl_type(
                  &introspection_input_value.type_ref.unwrap(),
                ),
                description: introspection_input_value.description,
                directives: vec![],
                name: introspection_input_value.name,
                position: Default::default(),
              })
              .collect(),
            field_type: introspection_output_typeref_to_sdl_type(&introspection_field.type_ref),
          })
          .collect(),
      }),
      IntrospectionType::INPUT_OBJECT(introspection_input_object) => {
        TypeDefinition::InputObject(InputObjectType {
          description: introspection_input_object.description,
          directives: vec![],
          name: introspection_input_object.name,
          position: Default::default(),
          fields: introspection_input_object
            .input_fields
            .into_iter()
            .map(|introspection_input_field| InputValue {
              default_value: introspection_input_field
                .default_value
                .map(serde_value_to_sdl_value),
              value_type: introspection_input_field
                .type_ref
                .map(|v| introspection_input_typeref_to_sdl_type(&v))
                .unwrap(),
              description: introspection_input_field.description,
              directives: vec![],
              name: introspection_input_field.name,
              position: Default::default(),
            })
            .collect(),
        })
      }
      IntrospectionType::SCALAR(introspection_scalar) => TypeDefinition::Scalar(ScalarType {
        position: Default::default(),
        name: introspection_scalar.name,
        description: introspection_scalar.description,
        directives: vec![],
      }),
      IntrospectionType::UNION(introspection_union) => TypeDefinition::Union(UnionType {
        position: Default::default(),
        name: introspection_union.name,
        description: introspection_union.description,
        directives: vec![],
        types: introspection_union
          .possible_types
          .into_iter()
          .map(|v| v.name)
          .collect(),
      }),
      IntrospectionType::ENUM(introspection_enum) => TypeDefinition::Enum(EnumType {
        position: Default::default(),
        name: introspection_enum.name,
        description: introspection_enum.description,
        directives: vec![],
        values: introspection_enum
          .enum_values
          .into_iter()
          .map(|introspection_enum_value| EnumValue {
            position: Default::default(),
            name: introspection_enum_value.name,
            description: introspection_enum_value.description,
            directives: vec![],
          })
          .collect(),
      }),
    };

    result
      .definitions
      .push(Definition::TypeDefinition(schema_type));
  }

  result
}

// Adapted from: https://github.com/graphql/graphql-js/blob/9c90a23dd430ba7b9db3d566b084e9f66aded346/src/utilities/getIntrospectionQuery.ts#L66
pub static INTROSPECTION_QUERY: &str = r#"query IntrospectionQuery {
  __schema {
    queryType { name }
    mutationType { name }
    subscriptionType { name }
    types {
      ...FullType
    }
    directives {
      name
      description
      locations
      args {
        ...InputValue
      }
    }
  }
}

fragment FullType on __Type {
  kind
  name
  description

  fields(includeDeprecated: true) {
    name
    description
    args {
      ...InputValue
    }
    type {
      ...TypeRef
    }
    isDeprecated
    deprecationReason
  }
  inputFields {
    ...InputValue
  }
  interfaces {
    ...TypeRef
  }
  enumValues(includeDeprecated: true) {
    name
    description
    isDeprecated
    deprecationReason
  }
  possibleTypes {
    ...TypeRef
  }
}

fragment InputValue on __InputValue {
  name
  description
  type { ...TypeRef }
  defaultValue
}

fragment TypeRef on __Type {
  kind
  name
  ofType {
    kind
    name
    ofType {
      kind
      name
      ofType {
        kind
        name
        ofType {
          kind
          name
          ofType {
            kind
            name
            ofType {
              kind
              name
              ofType {
                kind
                name
                ofType {
                  kind
                  name
                  ofType {
                    kind
                    name
                  }
                }
              }
            }
          }
        }
      }
    }
  }
}"#;
