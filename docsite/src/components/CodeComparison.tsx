import React from 'react';
import styles from './CodeComparison.module.css';
import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';
import CodeBlock from '@theme/CodeBlock';

type CodeComparisonProps = {
  ruby: string;
  rust: string;
  rubyTitle?: string;
  rustTitle?: string;
  rubyLanguage?: string;
  rustLanguage?: string;
};

/**
 * CodeComparison component
 *
 * A component that displays Ruby and Rust code side by side in tabs,
 * making it easy to compare implementations in both languages.
 */
export default function CodeComparison({
  ruby,
  rust,
  rubyTitle = 'Ruby',
  rustTitle = 'Rust',
  rubyLanguage = 'ruby',
  rustLanguage = 'rust',
}: CodeComparisonProps): React.ReactElement {
  return (
    <div className={styles.codeComparison}>
      <Tabs>
        <TabItem value="ruby" label={rubyTitle} default>
          <div className={styles.codeBlock}>
            <CodeBlock language={rubyLanguage}>
              {ruby}
            </CodeBlock>
          </div>
        </TabItem>
        <TabItem value="rust" label={rustTitle}>
          <div className={styles.codeBlock}>
            <CodeBlock language={rustLanguage}>
              {rust}
            </CodeBlock>
          </div>
        </TabItem>
      </Tabs>
    </div>
  );
}
