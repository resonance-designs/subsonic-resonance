#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const rootDir = path.resolve(__dirname, '..');
const docsDir = path.join(rootDir, 'docs-site', 'docs');

function read(relativePath) {
  const target = path.join(rootDir, relativePath);
  if (!fs.existsSync(target)) {
    throw new Error(`Required documentation source is missing: ${relativePath}`);
  }
  return fs.readFileSync(target, 'utf8').replace(/\r\n/g, '\n');
}

function section(markdown, heading) {
  const lines = markdown.split('\n');
  const start = lines.findIndex(line => line.trim() === `## ${heading}`);
  if (start < 0) throw new Error(`README.md does not contain the expected section: ${heading}`);
  let end = lines.length;
  for (let index = start + 1; index < lines.length; index += 1) {
    if (lines[index].startsWith('## ')) { end = index; break; }
  }
  return lines.slice(start + 1, end).join('\n').trim();
}

function withoutTitle(markdown) {
  return markdown.replace(/^\uFEFF?# .+\n+/, '').trim();
}

function generated(title, source, body, position) {
  return [
    '---',
    `title: ${title}`,
    `sidebar_position: ${position}`,
    '---',
    '',
    `<!-- Generated from ${source} by scripts/sync-docs.js. Do not edit directly. -->`,
    '',
    body.trim(),
    ''
  ].join('\n');
}

function write(relativePath, contents) {
  const target = path.join(docsDir, relativePath);
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.writeFileSync(target, contents);
}

function main() {
  const logoSource = path.join(rootDir, 'img', 'logo.png');
  const logoTarget = path.join(rootDir, 'docs-site', 'static', 'img', 'logo.png');
  const appAlbumsSource = path.join(rootDir, 'img', 'app-albums.png');
  const appAlbumsTarget = path.join(rootDir, 'docs-site', 'static', 'img', 'app-albums.png');
  if (!fs.existsSync(logoSource)) throw new Error('Required brand asset is missing: img/logo.png');
  if (!fs.existsSync(appAlbumsSource)) throw new Error('Required application screenshot is missing: img/app-albums.png');
  fs.mkdirSync(path.dirname(logoTarget), { recursive: true });
  fs.copyFileSync(logoSource, logoTarget);
  fs.copyFileSync(appAlbumsSource, appAlbumsTarget);

  const readme = read('README.md');
  const current = section(readme, 'Current functionality');
  const architecture = section(readme, 'Architecture');
  const prerequisites = section(readme, 'Prerequisites');
  const browser = section(readme, 'Run the browser client');
  const environment = section(readme, 'Optional default provider');
  const desktop = section(readme, 'Run the Windows desktop shell');
  const security = section(readme, 'Security and persistence');
  const checks = section(readme, 'Development checks');
  const roadmap = section(readme, 'Roadmap');
  const limitations = section(readme, 'Known limitations')
    .replace('[CHANGELOG.md](CHANGELOG.md)', '[Changelog](../releases/changelog)')
    .replace('[TODO.md](TODO.md)', '[implementation tracker](./todo)')
    .replace('[LICENSING.md](LICENSING.md)', '[licensing guide](./licensing)');

  write('intro.md', generated('Resonance', 'README.md', `# Resonance\n\n${readme.split('\n').slice(2, 5).join('\n').trim()}\n\n## Current functionality\n\n${current}`, 1));
  write('getting-started/installation.md', generated('Installation and development', 'README.md', `# Installation and development\n\n## Prerequisites\n\n${prerequisites}\n\n## Browser client\n\n${browser}\n\n## Optional default provider\n\n${environment}\n\n## Windows desktop shell\n\n${desktop}\n\n## Development checks\n\n${checks}`, 1));
  write('architecture/overview.md', generated('Architecture', 'README.md', `# Architecture\n\n${architecture}`, 1));
  write('project/status.md', generated('Project status', 'README.md', `# Project status\n\n## Current functionality\n\n${current}\n\n## Security and persistence\n\n${security}\n\n## Known limitations\n\n${limitations}`, 1));
  write('project/roadmap.md', generated('Roadmap', 'README.md', `# Roadmap\n\n${roadmap}`, 2));
  write('project/todo.md', generated('Implementation tracker', 'TODO.md', `# Implementation tracker\n\n${withoutTitle(read('TODO.md'))}`, 3));
  write('project/licensing.md', generated('Licensing guide', 'LICENSING.md', `# Licensing guide\n\n${withoutTitle(read('LICENSING.md'))}`, 4));
  write('releases/changelog.md', generated('Changelog', 'CHANGELOG.md', `# Changelog\n\n${withoutTitle(read('CHANGELOG.md'))}`, 1));

  console.log('Synchronized Resonance documentation into docs-site/docs.');
}

main();
