import clsx from 'clsx';
import Heading from '@theme/Heading';
import styles from './styles.module.css';
import React, { ReactNode } from 'react';

type FeatureItem = {
  title: string;
  Svg: React.ComponentType<React.ComponentProps<'svg'>>;
  description: ReactNode;
};

const FeatureList: FeatureItem[] = [
  {
    title: 'Easy to Use',
    Svg: require('@site/static/img/feature-easy-to-use.svg').default,
    description: (
      <>
        rb-sys integrates into Ruby projects with minimal setup,
        combining Ruby's flexibility with Rust's performance and safety.
      </>
    ),
  },
  {
    title: 'Focus on What Matters',
    Svg: require('@site/static/img/feature-focus.svg').default,
    description: (
      <>
        rb-sys handles the complex FFI integration between Ruby and Rust,
        letting you concentrate on your application code.
      </>
    ),
  },
  {
    title: 'Powered by Rust',
    Svg: require('@site/static/img/feature-powered-by-rust.svg').default,
    description: (
      <>
        Add Rust capabilities to Ruby applications with simplified
        bindings to the Ruby C API.
      </>
    ),
  },
];

function Feature({title, Svg, description}: FeatureItem) {
  return (
    <div className={clsx('col col--4')} style={{display: 'flex'}}>
      <div className={styles.featureItem}>
        <div className={styles.featureIconContainer}>
          <Svg className={styles.featureSvg} role="img" />
        </div>
        <div className={styles.featureContent}>
          <Heading as="h3" className={styles.featureTitle}>{title}</Heading>
          <div className={styles.featureDivider}></div>
          <div className={styles.featureDescription}>
            <p>{description}</p>
          </div>
        </div>
      </div>
    </div>
  );
}

export default function HomepageFeatures(): React.ReactElement {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="section-heading">
          <h2 className="section-title">Key Features</h2>
          <div className="section-divider"></div>
        </div>
        <div className="row" style={{display: 'flex', alignItems: 'stretch'}}>
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}
