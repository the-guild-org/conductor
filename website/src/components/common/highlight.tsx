import { PropsWithChildren, ReactElement } from "react";

export function Highlights(
  props: PropsWithChildren<{
    className?: string;
    items: {
      title: string;
      description?: string | ReactElement;
      icon: ReactElement;
    }[];
  }>
) {
  return (
    <div
      className={`flex flex-col md:flex-row gap-8 justify-between ${
        props.className || ""
      }`}
    >
      {props.items.map(({ title, description, icon }, i) => (
        <div
          key={i}
          className="flex flex-row md:flex-col lg:flex-row flex-1 gap-6"
        >
          <div className="text-teal-700 flex-shrink-0 flex items-center">
            {icon}
          </div>
          <div className="flex flex-col text-black dark:text-white">
            <h3 className="text-xl font-semibold">{title}</h3>
            {typeof description === "string" ? (
              <p className="text-gray-600 dark:text-gray-400">{description}</p>
            ) : (
              description
            )}
          </div>
        </div>
      ))}
    </div>
  );
}
