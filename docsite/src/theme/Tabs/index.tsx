import React, { useState, cloneElement, ReactElement, ReactNode } from "react";
import clsx from "clsx";
import styles from "./styles.module.css";

// Define types for TabItem props
interface TabItemProps {
  children: ReactNode;
  hidden?: boolean;
  className?: string;
  value: string;
  label: string;
}

// Export TabItem as a named export
export function TabItem({ children, hidden, className }: TabItemProps): React.ReactElement {
  return (
    <div role="tabpanel" className={clsx(styles.tabItem, className)} hidden={hidden}>
      {children}
    </div>
  );
}

// Define types for Tabs props
interface TabsProps {
  children: ReactNode;
  className?: string;
  defaultValue?: string;
}

function Tabs(props: TabsProps): React.ReactElement {
  const { children, className, defaultValue } = props;
  const childrenArray = React.Children.toArray(children).filter((child): child is ReactElement<TabItemProps> => {
    if (!React.isValidElement(child)) return false;
    const childProps = child.props as Partial<TabItemProps>;
    return "value" in childProps && "label" in childProps;
  });

  // Find the default tab index
  const defaultIndex = defaultValue ? childrenArray.findIndex((tabItem) => tabItem.props.value === defaultValue) : 0;

  const [selectedValue, setSelectedValue] = useState<string | null>(
    defaultIndex !== -1 ? childrenArray[defaultIndex].props.value : null,
  );

  const handleTabChange = (value: string): void => {
    setSelectedValue(value);
  };

  const tabLabels = childrenArray.map((tabItem) => ({
    value: tabItem.props.value,
    label: tabItem.props.label,
  }));

  return (
    <div className={clsx("tabs-container", styles.tabsContainer, className)}>
      <div role="tablist" aria-orientation="horizontal" className={styles.tabList}>
        {tabLabels.map(({ value, label }) => (
          <button
            role="tab"
            key={value}
            aria-selected={selectedValue === value}
            className={clsx(styles.tabItem, selectedValue === value && styles.tabItemActive)}
            onClick={() => handleTabChange(value)}
          >
            {label}
          </button>
        ))}
      </div>
      <div className={styles.tabContent}>
        {childrenArray.map((tabItem, i) =>
          cloneElement(tabItem, {
            key: i,
            hidden: selectedValue !== tabItem.props.value,
          }),
        )}
      </div>
    </div>
  );
}

// Attach TabItem as a property of Tabs
Tabs.TabItem = TabItem;

// Export Tabs as the default export
export default Tabs;
