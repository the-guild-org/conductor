import clsx from "clsx";
import * as mdxComponents from "nextra/components";
import { useMDXComponents } from "nextra/mdx";
import { PropsWithChildren, ReactElement, ReactNode } from "react";

const TAG_COLORS: Record<string, string> = {
  optional: "text-blue-400",
  required: "text-red-400",
  default: "text-purple-400",
  literal: "text-rose-900",
  enum: "text-yellow-400",
};

function Tag(props: { color: string; text: string | ReactElement }) {
  const color = TAG_COLORS[props.color] || "text-gray-400";

  return <div className={clsx(color, "px-2")}>{props.text}</div>;
}

function DocumentationContainer(
  props: PropsWithChildren<{
    tags: string[];
    title: string;
    subtitle?: string;
    collapsible?: boolean;
  }>
) {
  const rootMdx = useMDXComponents();
  const Summary = rootMdx.summary!;
  const Details = rootMdx.details!;
  const Code = rootMdx.code!;

  const tags = props.tags.map((raw) => {
    const [color, value] = raw.includes("|") ? raw.split("|") : [raw, raw];
    const text =
      color === value ? (
        value
      ) : (
        <span>
          {color}: <code>{value}</code>
        </span>
      );

    return <Tag color={color} text={text} key={color} />;
  });

  if (props.collapsible) {
    return (
      <Details className="border-0 mt-2">
        <Summary>
          <Code>{props.title}</Code>
          {props.subtitle}
          {tags}
        </Summary>
        <div className="pl-[33px]">{props.children}</div>
      </Details>
    );
  }

  return (
    <div className="first:mt-3 p-4 border border-neutral-400 dark:border-neutral-600 border-b-0 first:rounded-t-md last:rounded-b-md last:border-b">
      <div className="flex gap-2">
        <div>
          <Code>{props.title}</Code>
          {props.subtitle}
        </div>
        <div className="flex-1 flex gap-1 pb-1">{tags}</div>
      </div>
      <div>{props.children}</div>
    </div>
  );
}

export const components = {
  ...mdxComponents,
  DocumentationContainer,
};
