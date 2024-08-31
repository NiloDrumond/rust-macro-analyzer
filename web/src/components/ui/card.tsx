import { PropsWithChildren } from "react";
import { cn } from "../../utils/cn";

export function Card({
  className,
  children,
  ...props
}: PropsWithChildren<
  React.DetailedHTMLProps<React.HTMLAttributes<HTMLDivElement>, HTMLDivElement>
>) {
  return (
    <div
      className={cn(
        "flex flex-col border rounded-lg p-4 shadow-xl gap-2",
        className,
      )}
      {...props}
    >
      {children}
    </div>
  );
}
