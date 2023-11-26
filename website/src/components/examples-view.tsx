import { useState } from "react";
import dynamic from "next/dynamic";
import { Tabs } from "@theguild/components";

const ExampleEditor = dynamic({
  loader: () => import("./example-editor"),
  ssr: false,
});

const EXAMPLES = [
  {
    title: "Plugins",
    code: /* YAML */ `sources:
  - id: my-schema
    type: graphql
    config:
      endpoint: http://my-server.com/graphql

endpoints:
  - path: /graphql
    from: my-schema

  - path: /persisted
    from: my-schema
    plugins:
      - type: persisted_operations
        config:
          store:
            source: file
            path: store.json
            format: json_key_value
          protocols:
            - type: document_id
            - type: http_get
          allow_non_persisted: false

  - path: /dev
    from: my-schema
    plugins:
      - type: graphiql`,
  },
  {
    title: "Federated Schemas",
    code: /* YAML */ `sources:
    - id: my-supergraph
      type: supergraph
      config:
        file: supergraph.graphql
  
  endpoints:
    - path: /graphql
      from: my-schema`,
  },
];

export function ExamplesView() {
  const [activeExample, setActiveExample] = useState<number>(0);

  return (
    <div className="rounded-2xl overflow-hidden flex flex-grow flex-col">
      <div>
        <Tabs
          items={Object.values(EXAMPLES).map((v) => v.title)}
          onChange={(t) => setActiveExample(t)}
        >
          <div />
        </Tabs>
      </div>
      <ExampleEditor
        editorHeight="680px"
        lang="yaml"
        value={EXAMPLES[activeExample].code}
      />
    </div>
  );
}
