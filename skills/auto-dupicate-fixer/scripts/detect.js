#!/usr/bin/env node

/**
 * Phase 1: Detect Duplicates
 * Scans project for duplicate files, blocks, and structural similarities
 * Outputs: duplicates.json
 */

const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const { execSync } = require('child_process');

const projectRoot = process.argv[2] || process.cwd();
const format = process.argv[3] === '--format=json' ? 'json' : 'text';
const minTokens = parseInt(process.argv[4]?.split('=')[1] || '50');

const results = {
  timestamp: new Date().toISOString(),
  projectRoot,
  duplicates: [],
  structural: [],
  identical: [],
  metrics: {}
};

// Scan TS/JS files
function scanTypeScriptJavaScript() {
  const files = execSync(`find ${projectRoot} -type f \\( -name "*.ts" -o -name "*.tsx" -o -name "*.js" -o -name "*.jsx" \\) ! -path "*/node_modules/*" ! -path "*/.git/*"`, { encoding: 'utf-8' }).trim().split('\n').filter(Boolean);
  
  const fileHashes = {};
  const fileContents = {};

  files.forEach(file => {
    try {
      const content = fs.readFileSync(file, 'utf-8');
      const hash = crypto.createHash('md5').update(content).digest('hex');
      
      fileContents[file] = content;
      
      if (!fileHashes[hash]) {
        fileHashes[hash] = [];
      }
      fileHashes[hash].push(file);
    } catch (e) {
      console.error(`Error reading ${file}:`, e.message);
    }
  });

  // Identical files
  Object.entries(fileHashes).forEach(([hash, files]) => {
    if (files.length > 1) {
      results.identical.push({
        hash,
        files,
        size: fs.statSync(files[0]).size
      });
    }
  });

  // Structural similarity via jscpd
  try {
    const jscpdOutput = execSync(`npx jscpd --reporters=json --output ${projectRoot}/.jscpd-report.json ${projectRoot} --ignore=node_modules,dist,.git`, { encoding: 'utf-8', stdio: 'pipe' }).catch(() => '');
    const report = JSON.parse(fs.readFileSync(`${projectRoot}/.jscpd-report.json`, 'utf-8'));
    
    if (report.duplicates) {
      results.duplicates = report.duplicates.map(dup => ({
        files: dup.locations.map(loc => loc.file),
        lines: dup.lines,
        tokens: dup.tokens,
        percentage: dup.percentage
      })).filter(dup => dup.tokens >= minTokens);
    }
  } catch (e) {
    console.warn('jscpd not available, skipping block analysis');
  }

  results.metrics.tsJsFiles = files.length;
  results.metrics.identicalGroups = Object.keys(fileHashes).filter(h => fileHashes[h].length > 1).length;
}

// Scan Python files
function scanPython() {
  try {
    const pythonResult = execSync(`python3 scripts/detect.py "${projectRoot}" ${minTokens}`, { encoding: 'utf-8' });
    const pythonDuplicates = JSON.parse(pythonResult);
    results.python = pythonDuplicates;
  } catch (e) {
    console.warn('Python detection skipped:', e.message);
  }
}

// Main
console.log(`🔍 Scanning ${projectRoot} for duplicates...`);
scanTypeScriptJavaScript();
scanPython();

const outputFile = path.join(projectRoot, '.duplicate-report.json');
fs.writeFileSync(outputFile, JSON.stringify(results, null, 2));

if (format === 'json') {
  console.log(JSON.stringify(results, null, 2));
} else {
  console.log(`\n📊 Detection Summary:`);
  console.log(`  TS/JS Files: ${results.metrics.tsJsFiles}`);
  console.log(`  Identical Groups: ${results.metrics.identicalGroups}`);
  console.log(`  Duplicate Blocks: ${results.duplicates.length}`);
  console.log(`  Report: ${outputFile}`);
}

process.exit(0);
