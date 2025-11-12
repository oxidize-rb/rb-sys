import React from "react";
import { useThemeConfig } from "@docusaurus/theme-common";
import Link from "@docusaurus/Link";
import styles from "./styles.module.css";

function FooterLink({ to, href, label, prependBaseUrlToHref, ...props }) {
  const toUrl = to ? to : "";
  const targetLink = prependBaseUrlToHref ? href : href;
  const isExternalLink = href && (href.indexOf("http") === 0 || href.indexOf("mailto:") === 0);

  return (
    <Link
      className="footer__link-item"
      {...(href
        ? {
            target: isExternalLink ? "_blank" : undefined,
            rel: isExternalLink ? "noopener noreferrer" : undefined,
            href: targetLink,
          }
        : {
            to: toUrl,
          })}
      {...props}
    >
      {label}
      {isExternalLink && (
        <svg width="13.5" height="13.5" aria-hidden="true" viewBox="0 0 24 24" className={styles.iconExternalLink}>
          <path
            fill="currentColor"
            d="M21 13v10h-21v-19h12v2h-10v15h17v-8h2zm3-12h-10.988l4.035 4-6.977 7.07 2.828 2.828 6.977-7.07 4.125 4.172v-11z"
          />
        </svg>
      )}
    </Link>
  );
}

function FooterColumn({ title, items }) {
  return (
    <div className="footer__col">
      <div className="footer__title">{title}</div>
      <ul className="footer__items clean-list">
        {items.map((item, i) => (
          <li key={i} className="footer__item">
            <FooterLink {...item} />
          </li>
        ))}
      </ul>
    </div>
  );
}

function Footer() {
  const { footer } = useThemeConfig();
  const { copyright, links = [] } = footer || {};

  return (
    <footer className="footer">
      <div className="footer__container">
        {links && links.length > 0 && (
          <div className="footer__links">
            {links.map((linkItem, i) => (
              <div className="col" key={i}>
                {linkItem.title != null ? <FooterColumn title={linkItem.title} items={linkItem.items} /> : null}
              </div>
            ))}
            {/* Add a fourth empty column for visual balance */}
            <div className="col">
              <div className="footer__col">
                <div className="footer__title"></div>
                <ul className="footer__items clean-list"></ul>
              </div>
            </div>
          </div>
        )}
        {copyright && <div className="footer__copyright">{copyright}</div>}
      </div>
    </footer>
  );
}

export default Footer;
