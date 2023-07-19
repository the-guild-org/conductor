use graphql_parser::{
    parse_query, parse_schema,
    query::{Definition, Field, OperationDefinition, Selection},
    schema::{Document, TypeDefinition},
};
use std::collections::HashMap;
use std::error::Error;

pub struct SupergraphSchema<'a> {
    pub parsed_schema: Document<'a, String>,
    pub subgraph_schemas: HashMap<String, Document<'a, String>>,
}

pub struct QueryPlanner {
    pub schemas: HashMap<String, String>,
}

impl QueryPlanner {
    pub fn new(schemas: HashMap<String, String>) -> Self {
        QueryPlanner { schemas }
    }

    pub fn generate_query_plan(&self, query: &str) -> Result<QueryPlan, Box<dyn Error>> {
        let ast = parse_query(query)?;

        let mut steps = Vec::new();
        for definition in ast.definitions {
            match definition {
                Definition::Operation(op) => {
                    self.traverse_operation(op, &mut steps)?;
                }
                _ => {} // Handle other definition types...
            }
        }

        Ok(QueryPlan { steps })
    }

    fn traverse_operation(
        &self,
        operation: OperationDefinition<'_, String>,
        steps: &mut Vec<QueryStep>,
    ) -> Result<(), Box<dyn Error>> {
        match operation {
            OperationDefinition::Query(q) => {
                for selection in q.selection_set.items {
                    self.traverse_selection_set(
                        selection,
                        steps,
                        Operation::Query { fields: vec![] },
                    )?;
                }
            }
            OperationDefinition::Mutation(m) => {
                for selection in m.selection_set.items {
                    self.traverse_selection_set(
                        selection,
                        steps,
                        Operation::Mutation { fields: vec![] },
                    )?;
                }
            }
            _ => {} // Handle other operation types...
        }
        Ok(())
    }

    fn traverse_field(
        &self,
        field: &Field<String>,
        steps: &mut Vec<QueryStep>,
        kind: Operation,
    ) -> Result<(), Box<dyn Error>> {
        for selection in &field.selection_set.items {
            self.traverse_selection_set(selection.clone(), steps, kind.clone())?;
        }

        // Find which subgraph contains the current field.
        let subgraph = self
            .schemas
            .iter()
            .find(|(_, schema_doc)| self.field_exists_in_schema(&field.name, schema_doc))
            .map(|(name, _)| name.clone())
            .ok_or("Field not found in any subgraph")?;

        let operation = match kind {
            Operation::Query { mut fields } => {
                fields.push(field.name.clone());
                Operation::Query { fields }
            }
            Operation::Mutation { mut fields } => {
                fields.push(field.name.clone());
                Operation::Mutation { fields }
            }
        };

        steps.push(QueryStep {
            subgraph: subgraph,
            operation,
        });

        Ok(())
    }

    fn field_exists_in_schema(&self, field: &str, schema: &Document<'static, String>) -> bool {
        for def in &schema.definitions {
            if let graphql_parser::schema::Definition::TypeDefinition(TypeDefinition::Object(obj)) =
                def
            {
                for field_def in &obj.fields {
                    if field_def.name == field {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn traverse_selection_set(
        &self,
        selection: Selection<'_, String>,
        steps: &mut Vec<QueryStep>,
        kind: Operation,
    ) -> Result<(), Box<dyn Error>> {
        match selection {
            Selection::Field(field) => {
                self.traverse_field(&field, steps, kind.clone())?;
            }
            Selection::InlineFragment(inline_fragment) => {
                for selection in inline_fragment.selection_set.items {
                    self.traverse_selection_set(selection, steps, kind.clone())?;
                }
            }
            _ => {} // Handle other selection types...
        }
        Ok(())
    }
}

pub struct QueryPlan {
    steps: Vec<QueryStep>,
}

pub struct QueryStep {
    subgraph: String,
    operation: Operation,
}

#[derive(Clone)]
pub enum Operation {
    Query { fields: Vec<String> },
    Mutation { fields: Vec<String> },
}

#[cfg(test)]
mod tests {
    use super::*;
    use graphql_parser::parse_schema;
    use std::collections::HashMap;

    #[test]
    fn test_generate_query_plan() {
        let subgraph_schema1 = r#"
            type Query {
                foo: String
            }
        "#;

        let subgraph_schema2 = r#"
            type Query {
                bar: String
            }
        "#;

        let schema_str = r#"
            type Query {
                baz: String
            }
        "#;

        let parsed_schema: Document<'_, String> = parse_schema(schema_str).unwrap();

        let subgraph_schemas: HashMap<String, Document<String>> = vec![
            (
                "subgraph1".to_string(),
                parse_schema(subgraph_schema1).unwrap(),
            ),
            (
                "subgraph2".to_string(),
                parse_schema(subgraph_schema2).unwrap(),
            ),
        ]
        .into_iter()
        .collect();

        let supergraph_schema: HashMap<String, Document<'_, &str>> = HashMap::new();
        let query_planner = QueryPlanner {
            schemas: supergraph_schema,
        };
        let query = r#"{ bar, foo }"#;
        let result = query_planner.generate_query_plan(query);
        assert!(result.is_ok());
        let plan = result.unwrap();
        assert_eq!(plan.steps.len(), 1);
        assert_eq!(plan.steps[0].subgraph, "subgraph1"); // or "subgraph2", or "baz" - based on the query
        if let Operation::Query { fields } = &plan.steps[0].operation {
            assert_eq!(fields.len(), 1); // The query has only one field
        } else {
            panic!("Expected a Query operation");
        }
    }
}
