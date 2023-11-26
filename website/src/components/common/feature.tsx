import { PropsWithChildren, ReactElement } from "react";

export function Feature(
  props: PropsWithChildren<{
    title: string;
    description?: ReactElement;
    flipped?: boolean;
    side?: ReactElement;
    center?: boolean;
  }>
) {
  const { title, flipped } = props;

  return (
    <div
      className={`flex flex-col gap-12 ${
        props.center ? "items-center" : "items-start"
      } ${flipped ? `md:flex-row-reverse` : `md:flex-row`}`}
    >
      <div className="flex flex-col gap-4 w-full md:w-2/5 lg:w-1/2 flex-shrink-0">
        <h2
          className={`font-semibold text-5xl bg-clip-text leading-normal ${
            props.center ? "text-center" : ""
          }`}
        >
          {title}
        </h2>
        <div className="text-lg text-gray-600 dark:text-gray-400 leading-7">
          {props.description}
        </div>
      </div>
      {props.side ? (
        <div className="flex-grow flex mt-4">{props.side}</div>
      ) : null}
      {props.children}
    </div>
  );
}
