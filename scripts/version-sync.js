#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');

const rootDir = path.resolve(__dirname, '..');
const semver = /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/;

function read(relativePath) { return fs.readFileSync(path.join(rootDir, relativePath), 'utf8'); }
function write(relativePath, contents) { fs.writeFileSync(path.join(rootDir, relativePath), contents); }
function workspaceVersion() {
  const match = read('Cargo.toml').match(/\[workspace\.package\][\s\S]*?\bversion\s*=\s*"([^"]+)"/);
  if (!match) throw new Error('Unable to find [workspace.package] version in Cargo.toml.');
  return match[1];
}
function requestedVersion() {
  const version = process.argv[2] || workspaceVersion();
  if (!semver.test(version)) throw new Error(`Invalid semantic version: ${version}`);
  return version;
}
function replace(relativePath, pattern, replacement, optional = false) {
  const target = path.join(rootDir, relativePath);
  if (!fs.existsSync(target)) { if (optional) return; throw new Error(`Missing version target: ${relativePath}`); }
  const contents = fs.readFileSync(target, 'utf8');
  if (!pattern.test(contents)) { if (optional) return; throw new Error(`Version pattern not found in ${relativePath}`); }
  pattern.lastIndex = 0;
  write(relativePath, contents.replace(pattern, replacement));
}
function updateJson(relativePath, version) {
  const target = path.join(rootDir, relativePath);
  if (!fs.existsSync(target)) return;
  const json = JSON.parse(fs.readFileSync(target, 'utf8'));
  json.version = version;
  fs.writeFileSync(target, `${JSON.stringify(json, null, 2)}\n`);
}
function updatePackageLock(version) {
  const target = path.join(rootDir, 'package-lock.json');
  if (!fs.existsSync(target)) return;
  const lock = JSON.parse(fs.readFileSync(target, 'utf8'));
  lock.version = version;
  if (lock.packages && lock.packages['']) lock.packages[''].version = version;
  fs.writeFileSync(target, `${JSON.stringify(lock, null, 2)}\n`);
}
function ensureReleaseDoc(version) {
  const target = path.join(rootDir, 'docs-site', 'docs', 'releases', `${version}.md`);
  if (fs.existsSync(target)) return;
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.writeFileSync(target, `---\ntitle: Release ${version}\n---\n\n# Release ${version}\n\nSee the [full changelog](./changelog) for release details.\n`);
}
function run(command, args) {
  const result = spawnSync(command, args, { cwd: rootDir, stdio: 'inherit', shell: process.platform === 'win32' });
  if (result.status !== 0) throw new Error(`${command} ${args.join(' ')} failed.`);
}

function main() {
  const version = requestedVersion();
  replace('Cargo.toml', /(\[workspace\.package\][\s\S]*?\bversion\s*=\s*")[^"]+(")/, `$1${version}$2`);
  replace('crates/ui/Cargo.toml', /^version\s*=\s*"[^"]+"/m, `version = "${version}"`);
  updateJson('package.json', version);
  updatePackageLock(version);
  updateJson('docs-site/package.json', version);
  updateJson('docs-site/api/package.json', version);
  replace('docs-site/config/version.yml', /^version:\s*.*$/m, `version: '${version}'`);
  replace('docs-site/api/src/index.ts', /(\bversion:\s*['"])[^'"]+(['"])/, `$1${version}$2`, true);
  replace('README.md', /The project is currently at `[^`]+`/, `The project is currently at \`${version}\``);
  ensureReleaseDoc(version);
  run(process.execPath, ['scripts/sync-docs.js']);
  run('cargo', ['generate-lockfile']);
  console.log(`Synchronized Resonance version ${version} across Cargo, Node, and documentation files.`);
}

main();
