import type { ReactNode } from "react";
import clsx from "clsx";
import Link from "@docusaurus/Link";
import useDocusaurusContext from "@docusaurus/useDocusaurusContext";
import Layout from "@theme/Layout";
import HomepageFeatures from "@site/src/components/HomepageFeatures";
import Heading from "@theme/Heading";
import CodeBlock from "@theme/CodeBlock";
import Tabs from "@theme/Tabs";
import TabItem from "@theme/TabItem";

import styles from "./index.module.css";

function HomepageHeader() {
  const { siteConfig } = useDocusaurusContext();
  return (
    <header className={clsx("hero", styles.heroBanner)}>
      <div className={styles.heroTexture}></div>
      <div className={styles.heroOverlay}></div>
      <div className="container" style={{padding: 0, margin: '0 auto'}}>
        <div className={styles.heroContent}>
          <div className={styles.titleWrapper}>
            <div className={styles.titleAccent}></div>
            <Heading as="h1" className={styles.title}>
              {siteConfig.title}
            </Heading>
          </div>
          <p className={styles.subtitle}>{siteConfig.tagline}</p>
          <div className={styles.subtitleDivider}></div>
          <div className={styles.buttons}>
            <Link
              className={clsx("button button--lg", styles.primaryButton)}
              to="/docs/getting-started"
            >
              <span className={styles.buttonText}>Get Started</span>
              <span className={styles.buttonSheen}></span>
            </Link>
            <Link
              className={clsx("button button--lg", styles.secondaryButton)}
              to="/docs/project-setup"
            >
              <span className={styles.buttonText}>Installation Guide</span>
              <span className={styles.buttonSheen}></span>
            </Link>
          </div>
        </div>
      </div>
    </header>
  );
}

function HomepageCodeExample() {
  return (
    <section className={styles.codeExampleContainer}>
      <div className={styles.codeExampleBg}></div>
      <div className="container">
        <div className={styles.sectionHeading}>
          <h2 className={styles.sectionTitle}>Code Example</h2>
          <div className={styles.sectionDivider}></div>
          <p className={styles.sectionSubtitle}>Ruby extensions with Rust performance and safety</p>
        </div>
        <div className={styles.codeExample}>
          <div className={styles.codeExampleFrame}>
            <div className={styles.codeExampleLegend}>
              <div className={styles.codeLanguageBadge}><span className={styles.rubyDot}></span>Ruby</div>
              <div className={styles.codeLanguageBadge}><span className={styles.rustDot}></span>Rust</div>
            </div>
            <Tabs className={styles.codeTabs}>
              <TabItem value="ruby" label="Ruby">
                <div className={styles.codeBlockWrapper}>
                  <div className={styles.codeMeta}>Ruby Implementation</div>
                  <CodeBlock language="ruby" className={styles.codeBlock}>
                    {`# Define a Ruby class
class Calculator
  def add(a, b)
    a + b
  end
end

# Use it
calc = Calculator.new
puts calc.add(40, 2) # => 42`}
                  </CodeBlock>
                </div>
              </TabItem>
              <TabItem value="rust" label="Rust">
                <div className={styles.codeBlockWrapper}>
                  <div className={styles.codeMeta}>Rust Implementation</div>
                  <CodeBlock language="rust" className={styles.codeBlock}>
                    {`// Implement in Rust with rb-sys
use magnus::{define_class, function, method, prelude::*, Error};

fn add(a: i64, b: i64) -> i64 {
    a + b
}

#[magnus::init]
fn init() -> Result<(), Error> {
    let calculator = define_class("Calculator", Default::default())?;
    calculator.define_method("add", method!(add, 2))?;
    Ok(())
}`}
                  </CodeBlock>
                </div>
              </TabItem>
            </Tabs>
          </div>
        </div>
      </div>
    </section>
  );
}

export default function Home(): ReactNode {
  const { siteConfig } = useDocusaurusContext();
  return (
    <Layout title={siteConfig.title} description={siteConfig.tagline}>
      <HomepageHeader />
      <HomepageCodeExample />
      <main>
        <HomepageFeatures />
      </main>
    </Layout>
  );
}
