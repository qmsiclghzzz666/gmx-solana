import React, { ReactElement, ReactNode } from "react";

interface ExchangeInfoProps {
  children?: ReactNode;
  dividerClassName?: string;
  className?: string;
}

interface ExchangeInfoGroupProps {
  children?: ReactNode;
}

function ExchangeInfo({ children, className, dividerClassName = "line-divider" }: ExchangeInfoProps) {
  const childrenArr = React.Children.toArray(children) as ReactElement[];

  return (
    <div className={className}>
      {childrenArr
        .reduce((acc, child) => {
          if (isExchangeInfoGroup(child)) {
            const props = child.props as ExchangeInfoGroupProps;
            const groupChildren = React.Children.toArray(props.children).filter(Boolean) as React.ReactElement[];

            if (groupChildren.length) {
              acc.push(
                React.cloneElement(child, {
                  children: groupChildren,
                })
              );
            }
          } else {
            acc.push(child);
          }

          return acc;
        }, [] as React.ReactElement[])
        .map((child, index, arr) => {
          const isLast = index === arr.length - 1;

          return (
            <React.Fragment key={(child as ReactElement<{ key?: React.Key }>).props.key ?? index}>
              {child}
              {!isLast && isExchangeInfoGroup(child) && <div className={dividerClassName} />}
            </React.Fragment>
          );
        })}
    </div>
  );
}

function ExchangeInfoGroup({ children }: ExchangeInfoGroupProps) {
  return <>{children}</>;
}

function isExchangeInfoGroup(child: ReactNode) {
  return React.isValidElement(child) && child.type === ExchangeInfoGroup;
}

ExchangeInfo.Group = ExchangeInfoGroup;

export { ExchangeInfo };
