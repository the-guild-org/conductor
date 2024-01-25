import { PropsWithChildren, ReactElement } from 'react';

export function Feature(
  props: PropsWithChildren<{
    title: string;
    description?: ReactElement;
    flipped?: boolean;
    side?: ReactElement;
    center?: boolean;
    className?: string;
  }>,
) {
  const { title, flipped } = props;

  return (
    <div
      className={`flex flex-col gap-12 pb-20 ${props.center ? 'items-center' : 'items-start'} ${
        flipped ? `md:flex-row-reverse` : `md:flex-row`
      } ${props.className || ''}`}
    >
      <div className="flex w-full flex-shrink-0 flex-col gap-4 md:w-2/5 lg:w-1/2">
        <h2
          className={`bg-clip-text text-[32px] font-semibold leading-normal lg:text-5xl ${
            props.center ? 'text-center' : ''
          }`}
        >
          {title}
        </h2>
        <div className="text-gray-600 dark:text-gray-400 lg:text-lg lg:leading-7">
          {props.description}
        </div>
      </div>
      {props.side ? <div className="mt-4 flex w-full flex-grow">{props.side}</div> : null}
      {props.children}
    </div>
  );
}
