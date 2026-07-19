import React, { type ReactNode } from 'react';
import Link from '@docusaurus/Link';
import useBaseUrl from '@docusaurus/useBaseUrl';
import { useThemeConfig } from '@docusaurus/theme-common';
import ThemedImage from '@theme/ThemedImage';

/** Application-matched two-line brand lockup for the documentation navbar. */
export default function NavbarLogo(): ReactNode {
  const {
    navbar: { logo }
  } = useThemeConfig();
  const href = useBaseUrl(logo?.href || '/');
  const lightLogo = useBaseUrl(logo?.src || 'img/logo.png');
  const darkLogo = useBaseUrl(logo?.srcDark || logo?.src || 'img/logo.png');

  return (
    <Link className="navbar__brand resonance-navbar-brand" to={href}>
      <span className="navbar__logo resonance-navbar-brand__mark" aria-hidden="true">
        <ThemedImage
          sources={{ light: lightLogo, dark: darkLogo }}
          alt=""
          height={logo?.height}
          width={logo?.width}
        />
      </span>
      <span
        className="resonance-navbar-brand__wordmark"
        style={{
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'flex-start',
          justifyContent: 'center'
        }}>
        <small
          className="resonance-navbar-brand__subsonic"
          style={{ display: 'flex' }}>
          {'Subsonic'.split('').map((letter, index) => (
            <span key={`${letter}-${index}`}>{letter}</span>
          ))}
        </small>
        <strong
          className="resonance-navbar-brand__resonance"
          style={{ display: 'flex' }}>
          {'Resonance'.split('').map((letter, index) => (
            <span key={`${letter}-${index}`}>{letter}</span>
          ))}
        </strong>
      </span>
    </Link>
  );
}
