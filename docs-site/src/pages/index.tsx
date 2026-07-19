import type { ReactNode } from 'react';
import Link from '@docusaurus/Link';
import Layout from '@theme/Layout';
import useBaseUrl from '@docusaurus/useBaseUrl';
import { useThemeConfig } from '@docusaurus/theme-common';
import ThemedImage from '@theme/ThemedImage';

import styles from './index.module.css';

const HERO_LOGO_SIZE = 305;

export default function Home(): ReactNode {
  const {
    navbar: { logo }
  } = useThemeConfig();
  const lightLogo = useBaseUrl(logo?.src || 'img/logo.png');
  const darkLogo = useBaseUrl(logo?.srcDark || logo?.src || 'img/logo.png');
  const appAlbumsScreenshot = useBaseUrl('/img/app-albums.png');
  return (
    <Layout
      title="Subsonic Resonance documentation"
      description="Documentation for Subsonic Resonance, a unified OpenSubsonic streaming client."
    >
      <main>
        <header className={styles.heroBanner}>
          <div className={`container ${styles.heroGrid}`}>
            <div className={styles.heroCopy}>
              <div className={styles.heroIdentity}>
                <ThemedImage
                  className={styles.heroLogo}
                  sources={{ light: lightLogo, dark: darkLogo }}
                  alt="Subsonic Resonance logo"
                  height={HERO_LOGO_SIZE}
                  width={HERO_LOGO_SIZE}
                />
                <h1 className={`resonance-navbar-brand__wordmark ${styles.heroWordmark}`}>
                  <small className={`resonance-navbar-brand__subsonic ${styles.heroSubsonic}`}>
                    {'Subsonic'.split('').map((letter, index) => (
                      <span key={`${letter}-${index}`}>{letter}</span>
                    ))}
                  </small>
                  <strong className={`resonance-navbar-brand__resonance ${styles.heroResonance}`}>
                    {'Resonance'.split('').map((letter, index) => (
                      <span key={`${letter}-${index}`}>{letter}</span>
                    ))}
                  </strong>
                </h1>
              </div>
              <p className={styles.heroSubtitle}>
                A Unified OpenSubsonic Streaming Client. <br></br>
                One Library Across OpenSubsonic Servers, Bandcamp, and Local Sources.
              </p>
              <div className={styles.heroActions}>
                <Link className="button button--primary button--lg" to="/docs/intro">
                  Read Documentation
                </Link>
                <Link
                  className="button button--secondary button--lg"
                  href="https://github.com/resonance-designs/subsonic-resonance"
                >
                  View Repository
                </Link>
              </div>
            </div>
            <div className={styles.heroPreview}>
              <img
                className={styles.heroScreenshot}
                src={appAlbumsScreenshot}
                alt="Subsonic Resonance albums library interface"
              />
            </div>
          </div>
        </header>
      </main>
    </Layout>
  );
}
