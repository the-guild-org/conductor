import { PropsWithChildren, ReactElement } from 'react';

export function Feature(
  props: PropsWithChildren<{
    title: string;
    description?: ReactElement;
    flipped?: boolean;
    side?: ReactElement;
    center?: boolean;
  }>,
) {
  const { title, flipped } = props;

  return (
    <div
      className={`flex flex-col gap-12 ${props.center ? 'items-center' : 'items-start'} ${
        flipped ? `md:flex-row-reverse` : `md:flex-row`
      }`}
    >
      <div className="flex w-full flex-shrink-0 flex-col gap-4 md:w-2/5 lg:w-1/2">
        <h2
          className={`bg-clip-text text-5xl font-semibold leading-normal ${
            props.center ? 'text-center' : ''
          }`}
        >
          {title}
        </h2>
        <div className="text-lg leading-7 text-gray-600 dark:text-gray-400">
          {props.description}
        </div>
      </div>
      {props.side ? <div className="mt-4 flex flex-grow">{props.side}</div> : null}
      {props.children}
    </div>
  );
}
