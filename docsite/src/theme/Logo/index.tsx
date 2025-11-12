import React from "react";
import Link from "@docusaurus/Link";
import useBaseUrl from "@docusaurus/useBaseUrl";
import useDocusaurusContext from "@docusaurus/useDocusaurusContext";
import { useThemeConfig } from "@docusaurus/theme-common";
import ThemedImage from "@theme/ThemedImage";

interface LogoProps {
  className?: string;
  imageClassName?: string;
}

/**
 * Custom Logo component that uses the logo from config
 */
export default function Logo(props: LogoProps): React.ReactElement {
  const {
    siteConfig: { title },
  } = useDocusaurusContext();
  const themeConfig = useThemeConfig();
  const navbarTitle = themeConfig.navbar?.title || "";
  const navbarLogo = themeConfig.navbar?.logo?.src || null;

  const { className, imageClassName } = props;

  return (
    <Link
      to={useBaseUrl("/")}
      className={className}
      {...(navbarTitle
        ? {
            "aria-label": `${navbarTitle} home page`,
          }
        : {})}
    >
      {navbarLogo && (
        <img
          className={imageClassName}
          src={useBaseUrl(navbarLogo)}
          alt={themeConfig.navbar?.logo?.alt || "Logo"}
          width="32"
          height="32"
        />
      )}
      <span className="navbar__title">{navbarTitle || title}</span>
    </Link>
  );
}
