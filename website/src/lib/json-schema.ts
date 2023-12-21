// DOTAN: I'm terribly sorry for this code. I know it's bad. I know it's ugly. I know it's not maintainable.

import { GetStaticProps } from 'next';
import { JSONSchema7 } from 'json-schema';
import { stringify as jsonToYaml } from 'json-to-pretty-yaml';
import { buildDynamicMDX } from 'nextra/remote';
import rawSchema from '../../../libs/config/conductor.schema.json';

const configSchema = rawSchema as any as JSONSchema7;

type ExampleWithMetadata = Record<string, any> & {
  $metadata?: {
    title?: string;
    description?: string;
  };
};

function tryToResolveRef(root: JSONSchema7['definitions'], s: JSONSchema7): JSONSchema7 {
  if (s.$ref) {
    const name = s.$ref.replace('#/definitions/', '');
    const child = root![name];

    if (!child) {
      throw new Error(`Could not resolve reference ${name}`);
    }

    return child as JSONSchema7;
  }

  return s;
}

function makeDescription(description: string | undefined): string {
  if (!description) {
    return '';
  }

  const clean = description;

  return `<div className="mt-1 text-md">
${clean}
</div>`;
}

function visitDefinition(
  root: JSONSchema7['definitions'],
  definition: JSONSchema7,
  ignoreList: string[] = [],
): string {
  if (definition.type) {
    if (definition.type === 'object') {
      return (
        `<div>` +
        Object.entries(definition.properties || {})
          .map(([propName, rawDef]) => {
            const fieldDef = tryToResolveRef(root, rawDef as JSONSchema7);
            const tags: string[] = [];

            const definitionTypes = Array.isArray(fieldDef.type) ? fieldDef.type : [fieldDef.type];

            for (const type of definitionTypes) {
              if (type === 'null') {
                tags.push('optional');
              } else if (type && !fieldDef.enum) {
                tags.push(type);
              }
            }

            if (
              !tags.includes('optional') &&
              fieldDef.anyOf &&
              fieldDef.anyOf.length > 0 &&
              fieldDef.anyOf.find(v => (v as JSONSchema7).type === 'null')
            ) {
              tags.push('optional');
            }

            if (fieldDef.enum) {
              tags.push('enum');
            }

            const defaultValue = (rawDef as JSONSchema7).default || fieldDef.default;
            if (defaultValue !== undefined) {
              tags.push(`default|${JSON.stringify(defaultValue, null, 2)}`);
            }

            if (definition.required?.includes(propName)) {
              tags.push('required');
            }

            // This is a very special case where we have a field that is an enum with a single value.
            // This usually happens when we are using "untagged" of Serde.
            if (fieldDef.enum && fieldDef.enum.length === 1 && fieldDef.type === 'string') {
              return `<DocumentationContainer title={${JSON.stringify(propName)}} tags={["literal|${
                fieldDef.enum[0]
              }", "required"]}>
${makeDescription((rawDef as JSONSchema7).description || fieldDef.description)}
</DocumentationContainer>`;
            }

            return `<DocumentationContainer title={"${propName}"} tags={${JSON.stringify(tags)}}>
${makeDescription((rawDef as JSONSchema7).description || fieldDef.description)}
${ignoreList.includes(propName) ? '' : visitDefinition(root, fieldDef)}
</DocumentationContainer>`;
          })
          .join('\n') +
        '</div>'
      );
    } else if (definition.type === 'array') {
      const itemDefinition = tryToResolveRef(root, definition.items as JSONSchema7);

      const tags: string[] = [];

      if (definition.minItems) {
        tags.push(`min items: ${definition.minItems}`);
      }

      if (definition.maxItems) {
        tags.push(`max items: ${definition.maxItems}`);
      }

      return visitDefinition(root, itemDefinition);
    }
  } else if (definition.allOf) {
    return definition.allOf
      .map(item => {
        const itemDefinition = tryToResolveRef(root, item as JSONSchema7);

        return visitDefinition(root, itemDefinition);
      })
      .join('\n');
  } else if (definition.oneOf) {
    // Since simple enums are sometimes represented as oneOf, we want to show them as a block in a simple way.
    // This is because "schemars" does not use discriminators of JSONSchema.
    const showAsBlock = definition.oneOf.every(
      v =>
        (v as JSONSchema7).enum &&
        (v as JSONSchema7).enum?.length == 1 &&
        (v as JSONSchema7).type === 'string',
    );

    return [
      '<div className="font-bold pt-2 first:pt-0">The following options are valid for this field:</div>',
      '<div>',
      ...definition.oneOf.map(item => {
        const itemDefinition = tryToResolveRef(root, item as JSONSchema7);

        return `
<DocumentationContainer title={"${itemDefinition.title}"} tags={[]} ${
          showAsBlock ? '' : 'collapsible'
        }>
${makeDescription((item as JSONSchema7).description || itemDefinition.description)}
${visitDefinition(root, itemDefinition)}
</DocumentationContainer>`;
      }),
      '</div>',
    ].join('\n');
  } else if (definition.anyOf) {
    return definition.anyOf
      .map(item => {
        if ((item as JSONSchema7).type === 'null') {
          return '';
        }
        const itemDefinition = tryToResolveRef(root, item as JSONSchema7);

        return visitDefinition(root, itemDefinition);
      })
      .join('\n');
  }

  return '';
}

export function getStaticPropsFactory(
  entrypoint: undefined | string | null,
  title: string,
  ignoreList = [],
) {
  const schema =
    entrypoint === undefined
      ? null
      : entrypoint === null
        ? configSchema
        : ((configSchema.definitions! || [])[entrypoint] as JSONSchema7);

  if (!schema && entrypoint !== undefined) {
    throw new Error(`No definition found for entrypoint "${entrypoint}"`);
  }

  const getStaticProps: GetStaticProps = async ctx => {
    const blocks: string[] = [];
    blocks.push(`## ${title || schema?.title || entrypoint}`);

    if (schema?.description) {
      blocks.push(schema.description);
    }

    blocks.push(`## Configuration`);

    if (schema && schema.examples && Array.isArray(schema.examples) && schema.examples.length > 0) {
      blocks.push(`### Examples`);
      blocks.push(`<Tabs
      items={${JSON.stringify(
        schema.examples.map(
          (v, i) => (v as ExampleWithMetadata).$metadata?.title || `Example ${i + 1}`,
        ),
      )}}
    >
      ${schema.examples
        .map((v, i) => {
          const { $metadata, ...rest } = v as ExampleWithMetadata;

          return `<Tabs.Tab key={${i}}>
${$metadata?.description || ''}

<h3 className="mt-4 font-bold">YAML</h3>

\`\`\`yaml showLineNumbers
${jsonToYaml(rest).trim()}
\`\`\`

<h3 className="mt-4 font-bold">JSON</h3>

\`\`\`json showLineNumbers
${JSON.stringify(rest, null, 2)}
\`\`\`
          </Tabs.Tab>`;
        })
        .join('\n')}
    </Tabs>`);
    }

    if (schema) {
      blocks.push(`### Reference`);
      blocks.push(`<div>${visitDefinition(configSchema.definitions, schema, ignoreList)}</div>`);
    }

    const mdx = blocks.filter(Boolean).join('\n');
    const props = await buildDynamicMDX(mdx, {
      defaultShowCopyCode: true,
      codeHighlight: true,
      latex: false,
    });

    return { props };
  };

  return getStaticProps;
}
