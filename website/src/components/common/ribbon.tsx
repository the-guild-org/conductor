import { PropsWithChildren } from 'react';

export function Ribbon(props: PropsWithChildren) {
  return (
    <div className="float-right mr-9 mt-9 w-72 origin-top translate-x-1/2 rotate-45 bg-teal-900 py-2 pr-4 text-center text-white dark:text-white">
      {props.children}
    </div>
  );
}
