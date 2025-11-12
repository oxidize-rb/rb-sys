import React from "react";
import styles from "./LanguageCallout.module.css";
import clsx from "clsx";

type LanguageCalloutProps = {
  language: "ruby" | "rust";
  icon?: boolean;
  title?: string;
  children: React.ReactNode;
};

/**
 * LanguageCallout component
 *
 * A component that displays language-specific information in a callout box,
 * with appropriate styling for Ruby or Rust.
 */
export default function LanguageCallout({
  language,
  icon = true,
  title,
  children,
}: LanguageCalloutProps): React.ReactElement {
  const defaultTitle = language === "ruby" ? "Ruby" : "Rust";
  const displayTitle = title || defaultTitle;

  return (
    <div className={clsx(styles.languageCallout, styles[language])}>
      <div className={styles.header}>
        {icon && (
          <div className={styles.icon}>
            {language === "ruby" ? (
              // Ruby icon - simplified gem shape
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" height="20" width="20">
                <path d="M6 3h12l4 6-10 12L2 9l4-6z" strokeLinejoin="round" />
                <path d="M12 21L2 9h20" strokeLinejoin="round" />
              </svg>
            ) : (
              // Rust icon - simplified gear with crab claw
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" height="20" width="20">
                <circle cx="12" cy="12" r="8" />
                <path d="M12 4v2M12 18v2M4 12H6M18 12h2M6.34 6.34l1.42 1.42M16.24 16.24l1.42 1.42M6.34 17.66l1.42-1.42M16.24 7.76l1.42-1.42" />
                <path d="M9 12a3 3 0 1 0 6 0 3 3 0 0 0-6 0z" />
              </svg>
            )}
          </div>
        )}
        <div className={styles.title}>{displayTitle}</div>
      </div>
      <div className={styles.content}>{children}</div>
    </div>
  );
}
