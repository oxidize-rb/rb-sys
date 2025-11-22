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
      <div className="container" style={{ padding: 0, margin: "0 auto" }}>
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
            <Link className={clsx("button button--lg", styles.primaryButton)} to="/docs/getting-started">
              <span className={styles.buttonText}>Get Started</span>
              <span className={styles.buttonSheen}></span>
            </Link>
            <Link className={clsx("button button--lg", styles.secondaryButton)} to="/docs/project-setup">
              <span className={styles.buttonText}>Installation Guide</span>
              <span className={styles.buttonSheen}></span>
            </Link>
          </div>
        </div>
      </div>
    </header>
  );
}

function WhyRbSys() {
  return (
    <section className={styles.whyRbSys}>
      <div className="container">
        <div className={styles.sectionHeading}>
          <h2 className={styles.sectionTitle}>Why rb-sys?</h2>
          <div className={styles.sectionDivider}></div>
          <p className={styles.sectionSubtitle}>
            Leverage Rust's performance, safety, and modern tooling without leaving the Ruby ecosystem.
          </p>
        </div>
        <div className="row">
          <div className="col col--4">
            <h3>Performance</h3>
            <p>
              Write CPU-intensive code in Rust to speed up bottlenecks in your Ruby application. Ideal for parsing,
              image processing, and complex computations.
            </p>
          </div>
          <div className="col col--4">
            <h3>Memory Safety</h3>
            <p>
              Eliminate a whole class of bugs with Rust's compile-time memory safety guarantees. Say goodbye to
              segfaults and memory leaks from native extensions.
            </p>
          </div>
          <div className="col col--4">
            <h3>Modern Tooling</h3>
            <p>
              Get access to the entire Rust ecosystem, including Cargo, a first-class package manager, and a rich
              library of existing crates.
            </p>
          </div>
        </div>
      </div>
    </section>
  );
}

function HomepageCodeExample() {
  return (
    <section className={styles.codeExampleContainer}>
      <div className={styles.codeExampleBg}></div>
      <div className="container">
        <div className={styles.sectionHeading}>
          <h2 className={styles.sectionTitle}>High-Performance JSON Parsing</h2>
          <div className={styles.sectionDivider}></div>
          <p className={styles.sectionSubtitle}>
            A real-world example of replacing a pure Ruby method with a much faster Rust implementation.
          </p>
        </div>
        <div className={styles.codeExample}>
          <div className={styles.codeExampleFrame}>
            <div className={styles.codeExampleLegend}>
              <div className={styles.codeLanguageBadge}>
                <span className={styles.rubyDot}></span>Ruby
              </div>
              <div className={styles.codeLanguageBadge}>
                <span className={styles.rustDot}></span>Rust
              </div>
            </div>
            <Tabs className={styles.codeTabs}>
              <TabItem value="ruby" label="Ruby">
                <div className={styles.codeBlockWrapper}>
                  <div className={styles.codeMeta}>Ruby's standard JSON library</div>
                  <CodeBlock language="ruby" className={styles.codeBlock}>
                    {`require 'json'

json_string = '{"name": "John Doe", "age": 30, "is_student": false}'

# Parse with standard library
parsed_data = JSON.parse(json_string)

puts parsed_data["name"] # => "John Doe"`}
                  </CodeBlock>
                </div>
              </TabItem>
              <TabItem value="rust" label="Rust">
                <div className={styles.codeBlockWrapper}>
                  <div className={styles.codeMeta}>Rust implementation with rb-sys & serde</div>
                  <CodeBlock language="rust" className={styles.codeBlock}>
                    {`use magnus::{function, prelude::*, Error, Ruby, Value};

// Helper to convert serde_json::Value to Magnus's Value
fn json_to_ruby(ruby: &Ruby, value: serde_json::Value) -> Result<Value, Error> {
    // ... implementation details ...
    magnus::serde::to_value(ruby, &value)
}

// The high-performance parsing function
fn parse_json(ruby: &Ruby, json_string: String) -> Result<Value, Error> {
    let value: serde_json::Value = serde_json::from_str(&json_string)
        .map_err(|e| Error::new(magnus::exception::runtime_error(), e.to_string()))?;

    json_to_ruby(ruby, value)
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("FastJson")?;
    module.define_singleton_method("parse", function!(parse_json, 1))?;
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

function UsedBy() {
  return (
    <section className={styles.usedBy}>
      <div className="container">
        <div className={styles.sectionHeading}>
          <h2 className={styles.sectionTitle}>Used By</h2>
          <div className={styles.sectionDivider}></div>
          <p className={styles.sectionSubtitle}>
            `rb-sys` and the oxidize-rb toolchain are trusted in production by these and other great projects.
          </p>
        </div>
        <div className={styles.usedByLogos}>
          {/* Placeholder logos. In a real scenario, you'd have SVGs or images. */}
          <div className={styles.logoItem}>
            <a href="https://github.com/bytecodealliance/wasmtime-rb">wasmtime-rb</a>
          </div>
          <div className={styles.logoItem}>
            <a href="https://github.com/oxidize-rb/blake3-ruby">blake3-ruby</a>
          </div>
          <div className={styles.logoItem}>
            <a href="https://github.com/yoshoku/lz4-ruby">lz4-ruby</a>
          </div>
        </div>
      </div>
    </section>
  );
}

export default function Home(): ReactNode {
  const { siteConfig } = useDocusaurusContext();
  return (
    <Layout title="Building Ruby extensions with Rust" description={siteConfig.tagline}>
      <HomepageHeader />
      <main>
        <WhyRbSys />
        <HomepageFeatures />
        <HomepageCodeExample />
        <UsedBy />
      </main>
    </Layout>
  );
}
