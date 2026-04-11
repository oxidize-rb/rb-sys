import React, { useCallback } from "react";
import { useThemeConfig } from "@docusaurus/theme-common";
import { splitNavbarItems, useNavbarMobileSidebar } from "@docusaurus/theme-common/internal";
import NavbarItem from "@theme/NavbarItem";
import NavbarLayout from "@theme/Navbar/Layout";
import NavbarLogo from "@theme/Logo";
import styles from "./styles.module.css";

interface NavbarItemType {
  label?: string;
  to?: string;
  href?: string;
  className?: string;
  [key: string]: unknown;
}

interface NavbarItemsProps {
  items: NavbarItemType[] | undefined;
}

function NavbarItems({ items }: NavbarItemsProps): React.ReactElement {
  return <>{items && items.map ? items.map((item, i) => <NavbarItem {...item} key={i} />) : null}</>;
}

export default function Navbar(): React.ReactElement {
  const {
    navbar: { items },
  } = useThemeConfig();

  const mobileSidebar = useNavbarMobileSidebar();

  const splitItems = splitNavbarItems(items);
  const leftItems = splitItems[0];
  const rightItems = splitItems[1];

  // Custom handler to toggle the sidebar
  const handleSidebarToggle = useCallback(() => {
    mobileSidebar.toggle();
  }, [mobileSidebar]);

  return (
    <NavbarLayout>
      <div className="navbar__inner">
        <div className="navbar__items">
          <button
            aria-label="Navigation bar toggle"
            className="navbar__toggle clean-btn"
            type="button"
            tabIndex={0}
            onClick={handleSidebarToggle}
          >
            <svg width="30" height="30" viewBox="0 0 30 30" aria-hidden="true">
              <path
                stroke="currentColor"
                strokeLinecap="round"
                strokeMiterlimit="10"
                strokeWidth="2"
                d="M4 7h22M4 15h22M4 23h22"
              />
            </svg>
          </button>
          <NavbarLogo className="navbar__brand" />
          <NavbarItems items={leftItems} />
        </div>
        <div className="navbar__items navbar__items--right">
          <NavbarItems items={rightItems} />
        </div>
      </div>
    </NavbarLayout>
  );
}
