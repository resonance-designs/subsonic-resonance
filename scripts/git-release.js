#!/usr/bin/env node

const { execFileSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const repoRoot = path.resolve(__dirname, '..');
const dryRun = process.argv.includes('--dry-run');
const skipChecks = process.argv.includes('--skip-checks');

function run(command, args, options = {}) {
  const output = execFileSync(command, args, {
    cwd: repoRoot,
    encoding: 'utf8',
    stdio: options.inherit ? 'inherit' : ['ignore', 'pipe', 'pipe'],
    shell: false
  });
  return typeof output === 'string' ? output.trim() : '';
}

function git(args, options) {
  return run('git', args, options);
}

function read(relativePath) {
  return fs.readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

function workspaceVersion() {
  const match = read('Cargo.toml').match(/\[workspace\.package\][\s\S]*?\bversion\s*=\s*"([^"]+)"/);
  if (!match || !/^\d+\.\d+\.\d+$/.test(match[1])) {
    throw new Error('Cargo.toml must contain a stable x.y.z [workspace.package] version.');
  }
  return match[1];
}

function versionFromJson(relativePath) {
  return JSON.parse(read(relativePath)).version;
}

function ensureVersionsMatch(version) {
  const versions = new Map([
    ['package.json', versionFromJson('package.json')],
    ['docs-site/package.json', versionFromJson('docs-site/package.json')],
    ['docs-site/api/package.json', versionFromJson('docs-site/api/package.json')]
  ]);
  const uiMatch = read('crates/ui/Cargo.toml').match(/^version\s*=\s*"([^"]+)"/m);
  const docsMatch = read('docs-site/config/version.yml').match(/^version:\s*['"]?([^'"\s]+)['"]?/m);
  versions.set('crates/ui/Cargo.toml', uiMatch?.[1]);
  versions.set('docs-site/config/version.yml', docsMatch?.[1]);

  const mismatches = [...versions].filter(([, value]) => value !== version);
  if (mismatches.length) {
    const details = mismatches.map(([file, value]) => `${file}: ${value || 'missing'}`).join(', ');
    throw new Error(`Version metadata does not match Cargo ${version}: ${details}. Run npm run version:sync -- ${version}.`);
  }
  if (!read('CHANGELOG.md').includes(`## [${version}]`)) {
    throw new Error(`CHANGELOG.md does not contain a ${version} release entry.`);
  }
}

function currentBranch() {
  const branch = git(['branch', '--show-current']);
  if (!branch) throw new Error('Cannot release from a detached HEAD.');
  return branch;
}

function ensureCleanWorktree() {
  const status = git(['status', '--porcelain']);
  if (status) throw new Error('Working tree has uncommitted changes. Commit or stash them before releasing.');
}

function refExists(args) {
  try { git(args); return true; } catch { return false; }
}

function ensureOrigin() {
  try { git(['remote', 'get-url', 'origin']); }
  catch { throw new Error('The repository does not have an origin remote.'); }
}

function ensureRefsAvailable(branchName, tagName) {
  if (refExists(['show-ref', '--verify', '--quiet', `refs/heads/${branchName}`])) {
    throw new Error(`Local branch ${branchName} already exists.`);
  }
  if (refExists(['show-ref', '--verify', '--quiet', `refs/tags/${tagName}`])) {
    throw new Error(`Local tag ${tagName} already exists.`);
  }
  if (refExists(['ls-remote', '--exit-code', '--heads', 'origin', branchName])) {
    throw new Error(`Remote branch origin/${branchName} already exists.`);
  }
  if (refExists(['ls-remote', '--exit-code', '--tags', 'origin', `refs/tags/${tagName}`])) {
    throw new Error(`Remote tag ${tagName} already exists.`);
  }
}

function runReleaseChecks() {
  if (skipChecks) {
    console.log('Skipping build and test checks by request.');
    return;
  }
  console.log('Running Rust workspace tests...');
  run('cargo', ['test', '--workspace'], { inherit: true });
  console.log('Building documentation...');
  if (process.platform === 'win32') {
    run('cmd.exe', ['/d', '/s', '/c', 'npm run docs:build'], { inherit: true });
  } else {
    run('npm', ['run', 'docs:build'], { inherit: true });
  }
}

function main() {
  const version = workspaceVersion();
  const sourceBranch = currentBranch();
  const releaseBranch = `release/${version}`;
  const tagName = `v${version}`;

  ensureOrigin();
  ensureVersionsMatch(version);
  ensureRefsAvailable(releaseBranch, tagName);

  console.log(`Source branch:  ${sourceBranch}`);
  console.log(`Release branch: ${releaseBranch}`);
  console.log(`Release tag:    ${tagName}`);

  if (dryRun) {
    const dirty = git(['status', '--porcelain']);
    console.log(dirty ? 'Dry run: worktree is currently dirty; an actual release would stop.' : 'Dry run: worktree is clean.');
    console.log('Dry run complete. No branch, tag, commit, or remote was changed.');
    return;
  }

  ensureCleanWorktree();
  runReleaseChecks();
  ensureCleanWorktree();

  console.log(`Creating ${releaseBranch} from ${sourceBranch}...`);
  git(['switch', '-c', releaseBranch], { inherit: true });
  console.log(`Creating annotated tag ${tagName}...`);
  git(['tag', '-a', tagName, '-m', `Release ${version}`], { inherit: true });
  console.log('Publishing branch and tag atomically...');
  git(['push', '--atomic', 'origin', releaseBranch, tagName], { inherit: true });
  git(['branch', '--set-upstream-to', `origin/${releaseBranch}`, releaseBranch], { inherit: true });
  console.log(`Release published: ${releaseBranch} at ${tagName}`);
}

try {
  main();
} catch (error) {
  console.error(`Release failed: ${error.message}`);
  process.exitCode = 1;
}
