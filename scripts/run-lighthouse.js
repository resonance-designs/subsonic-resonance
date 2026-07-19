#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const lighthouseModule = require('lighthouse');
const chromeLauncher = require('chrome-launcher');
const lighthouse = lighthouseModule.default || lighthouseModule;

const url = process.argv[2] || 'http://127.0.0.1:8080';
const minimumAccessibility = Number(process.env.LIGHTHOUSE_MIN_ACCESSIBILITY || '0.90');

async function main() {
  let chrome;
  try {
    chrome = await chromeLauncher.launch({ chromeFlags: ['--headless=new', '--no-sandbox', '--disable-dev-shm-usage'] });
    const result = await lighthouse(url, {
      logLevel: 'error', output: 'json', port: chrome.port,
      onlyCategories: ['accessibility', 'best-practices', 'seo']
    });
    const report = result.lhr;
    const outputDir = path.resolve(__dirname, '..', 'artifacts', 'lighthouse');
    fs.mkdirSync(outputDir, { recursive: true });
    const safeName = new URL(url).host.replace(/[^a-z0-9.-]/gi, '_');
    const reportPath = path.join(outputDir, `${safeName}.json`);
    fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));
    const scores = {
      accessibility: report.categories.accessibility?.score ?? 0,
      bestPractices: report.categories['best-practices']?.score ?? 0,
      seo: report.categories.seo?.score ?? 0
    };
    const failures = Object.values(report.audits)
      .filter(audit => audit.score !== null && audit.score < 1 && audit.scoreDisplayMode !== 'notApplicable')
      .filter(audit => ['accessibility', 'best-practices'].some(category => audit.details || report.categories[category]?.auditRefs?.some(ref => ref.id === audit.id)))
      .map(audit => ({ id: audit.id, score: audit.score, title: audit.title }));
    console.log(JSON.stringify({ status: scores.accessibility >= minimumAccessibility ? 'ok' : 'failed', url, minimumAccessibility, scores, failures, reportPath }, null, 2));
    if (scores.accessibility < minimumAccessibility) process.exitCode = 1;
  } catch (error) {
    console.error(JSON.stringify({ status: 'error', url, message: error.message }, null, 2));
    process.exitCode = 1;
  } finally {
    if (chrome) await chrome.kill();
  }
}

main();
