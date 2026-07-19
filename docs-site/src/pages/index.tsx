import type { ReactNode } from 'react';
import Link from '@docusaurus/Link';
import Layout from '@theme/Layout';
import Heading from '@theme/Heading';

import styles from './index.module.css';

export default function Home(): ReactNode {
  return (
    <Layout
      title="Resonance documentation"
      description="Documentation for the Resonance unified OpenSubsonic streaming client"
    >
      <main>
        <header className={styles.heroBanner}>
          <div className="container">
            <p>RESONANCE DESIGNS</p>
            <Heading as="h1" className="hero__title">
              Resonance
            </Heading>
            <p className={styles.heroSubtitle}>
              One library across OpenSubsonic servers, Bandcamp, and future local sources.
            </p>
            <div style={{ display: 'flex', gap: '1rem', flexWrap: 'wrap', marginTop: '2rem' }}>
              <Link className="button button--primary button--lg" to="/docs/intro">
                Read the documentation
              </Link>
              <Link
                className="button button--secondary button--lg"
                href="https://github.com/resonance-designs/subsonic-resonance"
              >
                View the repository
              </Link>
            </div>
          </div>
        </header>
      </main>
    </Layout>
  );
}
