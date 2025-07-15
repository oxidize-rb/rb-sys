import React, { ReactNode } from 'react';
import clsx from 'clsx';
import DocItemContent from '@theme/DocItem/Content';
import styles from './styles.module.css';

interface DocItemLayoutProps {
  children: ReactNode;
}

export default function DocItemLayout({ children }: DocItemLayoutProps): React.ReactElement {
  return (
    <div className="row">
      <div className={clsx('col', styles.docItemCol)}>
        <div className={styles.docItemContainer}>
          <article>
            <DocItemContent>{children}</DocItemContent>
          </article>
        </div>
      </div>
    </div>
  );
}
