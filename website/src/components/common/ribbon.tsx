import { PropsWithChildren } from "react";

export function Ribbon(props: PropsWithChildren) {
  return (
    <div className="text-white dark:text-white py-2 pr-4 bg-teal-900 origin-top float-right mt-9 mr-9 w-72 text-center translate-x-1/2 rotate-45">
      {props.children}
    </div>
  );
}
