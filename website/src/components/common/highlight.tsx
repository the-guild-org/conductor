import { PropsWithChildren, ReactElement } from 'react';

export function Highlights(
  props: PropsWithChildren<{
    className?: string;
    items: {
      title: string;
      description?: string | ReactElement;
      icon: ReactElement;
    }[];
  }>,
) {
  return (
    <div className={`flex flex-col justify-between gap-8 md:flex-row ${props.className || ''}`}>
      {props.items.map(({ title, description, icon }, i) => (
        <div key={i} className="flex flex-1 flex-row gap-6 md:flex-col lg:flex-row">
          <div className="flex flex-shrink-0 items-center text-teal-700">{icon}</div>
          <div className="flex flex-col text-black dark:text-white">
            <h3 className="text-xl font-semibold">{title}</h3>
            {typeof description === 'string' ? (
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
