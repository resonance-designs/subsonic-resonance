import React from 'react';
import Layout from '@theme/Layout';
import Link from '@docusaurus/Link';

const demos = [
  { href: '/demos/badges', label: 'Badges' },
  { href: '/demos/comments', label: 'Comments' },
  { href: '/demos/cv', label: 'CV' },
  { href: '/demos/feature-flags', label: 'Feature Flags' },
  { href: '/demos/github-links', label: 'GitHub Links' },
  { href: '/demos/portfolio', label: 'Portfolio' },
  { href: '/demos/projects', label: 'Projects' },
  { href: '/demos/reader-mode', label: 'Reader Mode' },
  { href: '/demos/related-resources', label: 'Related Resources' },
  { href: '/demos/schema-system', label: 'Schema System' },
  { href: '/demos/text-size-switcher', label: 'Text Size Switcher' },
  { href: '/demos/theme-switcher', label: 'Theme Switcher' },
  { href: '/demos/version-display', label: 'Version Display' },
  { href: '/demos/404', label: '404 Demo' }
];

export default function DemosIndex(): React.JSX.Element {
  return (
    <Layout
      title="Demo Index"
      description="RaySon docs-site demo pages for template components and integrations"
    >
      <main className="container margin-top--lg margin-bottom--lg">
        <header className="margin-bottom--lg">
          <h1>Demo Index</h1>
          <p>Browse the available demo pages used in this docs site.</p>
        </header>

        <div className="row">
          <div className="col col--10 col--offset-1">
            <div className="button-group button-group--block">
              {demos.map((demo) => (
                <Link
                  key={demo.href}
                  to={demo.href}
                  className="button button--secondary margin-bottom--sm"
                >
                  {demo.label}
                </Link>
              ))}
            </div>
          </div>
        </div>
      </main>
    </Layout>
  );
}
