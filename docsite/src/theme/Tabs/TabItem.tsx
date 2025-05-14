import React, { ReactNode } from "react";
import clsx from "clsx";
import styles from "./styles.module.css";

export interface TabItemProps {
  children: ReactNode;
  hidden?: boolean;
  className?: string;
  value: string;
  label: string;
  default?: boolean;
  [key: string]: unknown;
}

export default function TabItem({
  children,
  hidden,
  className,
  value,
  label,
  default: isDefault,
  ...props
}: TabItemProps): React.ReactElement {
  return (
    <div
      role="tabpanel"
      className={clsx(styles.tabItem, className)}
      hidden={hidden}
      {...props}
    >
      {children}
    </div>
  );
}
