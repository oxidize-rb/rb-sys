import React, { ReactNode } from 'react';
import clsx from 'clsx';
import ErrorBoundary from '@docusaurus/ErrorBoundary';
import { PageMetadata, ThemeClassNames } from '@docusaurus/theme-common';
import { useKeyboardNavigation } from '@docusaurus/theme-common/internal';
import SkipToContent from '@theme/SkipToContent';
import AnnouncementBar from '@theme/AnnouncementBar';
import Navbar from '@theme/Navbar';
import Footer from '@theme/Footer';
import LayoutProvider from '@theme/Layout/Provider';
import styles from './styles.module.css';

interface LayoutProps {
  children: ReactNode;
  noFooter?: boolean;
  wrapperClassName?: string;
  title?: string;
  description?: string;
}

export default function Layout(props: LayoutProps): React.ReactElement {
  const {
    children,
    noFooter,
    wrapperClassName,
    // Not really layout-related, but kept for compatibility
    title,
    description,
  } = props;

  useKeyboardNavigation();

  return (
    <LayoutProvider>
      <PageMetadata title={title} description={description} />

      <SkipToContent />

      <AnnouncementBar />

      <Navbar />

      <div className={clsx(ThemeClassNames.wrapper.main, wrapperClassName, styles.mainWrapper)}>
        <ErrorBoundary fallback={(params) => <div>Error: {params.error.message}</div>}>
          <main className={styles.mainContent}>{children}</main>
        </ErrorBoundary>
      </div>

      {!noFooter && <Footer />}
    </LayoutProvider>
  );
}
