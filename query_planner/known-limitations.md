# Query Planner V1 Current Limitations

### Type Merging

- It can't accurately associate entities respones with their parent fields using the key fields (medium)
- Fields aliases aren't yet implemented (easy)
- Unwanted fields aren't filtered (easy)

### Query Planner

- Nested entities results in nested _entities queries, which it should not, it should extract it into its own query as soon as it detects an entity within another entity  (easy)
- 2+ nested fields' children of different subgraphs are treated to be of the same subgraph as the parent field  (difficult)
- It does a lot of supergraph lookups which I don't like  (medium)
- Parallel vs Sequential blocks decisions can sometimes not accurately detect sub-queries that can be done in paralel (easy)
- It can capture @inaccessible, requires and provides, but don't take action upon their existance (easy)
- automatically populated nested key selection sets for entities' `@key` don't function (easy)

### Execution
- Doesn't cach repeatedly requested fields -- Data Loaders (medium - difficult)
- Bubbles only a single error, and it can be an internal error sometimes which shouldn't be exposed, it should store a vector/list of errors as it's exeucting and provide the user with comprehensive error messages.  (medium)
