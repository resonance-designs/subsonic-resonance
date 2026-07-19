# Resonance documentation site

This Docusaurus application publishes the documentation for the Resonance OpenSubsonic streaming client. It is based on the MIT-licensed [Resonance Docusaurus template](https://github.com/resonance-designs/resonance-docusaurus); the upstream license is retained in this directory.

Project documentation is generated from the repository-level `README.md`, `TODO.md`, `CHANGELOG.md`, and `LICENSING.md`. Do not edit generated files under `docs/project`, `docs/architecture`, `docs/getting-started`, or `docs/releases/changelog.md` directly.

From the repository root:

```powershell
npm run docs:sync
npm run docs:start
npm run docs:build
npm run docs:quality
```

The development server uses `http://127.0.0.1:3000` by default. Production output is written to `docs-site/artifacts` and is excluded from version control.

Run `npm run lighthouse:docs` while the documentation server is running to produce an accessibility, best-practices, and SEO report.
