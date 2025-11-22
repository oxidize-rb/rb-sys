import React from "react";
import clsx from "clsx";
import { useWindowSize } from "@docusaurus/theme-common";
import { useDoc } from "@docusaurus/plugin-content-docs/client";
import DocItemPaginator from "@theme/DocItem/Paginator";
import DocVersionBanner from "@theme/DocVersionBanner";
import DocVersionBadge from "@theme/DocVersionBadge";
import DocItemFooter from "@theme/DocItem/Footer";
import DocItemTOCMobile from "@theme/DocItem/TOC/Mobile";
import DocItemTOCDesktop from "@theme/DocItem/TOC/Desktop";
import DocItemContent from "@theme/DocItem/Content";
import DocBreadcrumbs from "@theme/DocBreadcrumbs";
import CopyMarkdownButton from "@site/src/components/CopyMarkdownButton";
import styles from "./styles.module.css";
/**
 * A react component that is used to render a layout for a doc item.
 *
 * @param {object} props - The props for the component.
 * @param {React.ReactNode} props.children - The children to render.
 * @returns {React.ReactElement} The rendered component.
 */
export default function DocItemLayout({ children }) {
  const docElement = (
    <div className={styles.docItemContainer}>
      <article>
        <div className={styles.headerContainer}>
          <DocBreadcrumbs />
          <CopyMarkdownButton className={styles.copyButton} />
        </div>
        <DocVersionBadge />
        <DocItemTOCMobile />
        <DocItemContent>{children}</DocItemContent>
        <DocItemFooter />
      </article>
      <DocItemPaginator />
    </div>
  );

  return (
    <div className="row">
      <div className={clsx("col", !true && styles.docItemCol)}>
        <DocVersionBanner />
        {docElement}
      </div>
      <div className="col col--3">
        <DocItemTOCDesktop />
      </div>
    </div>
  );
}
