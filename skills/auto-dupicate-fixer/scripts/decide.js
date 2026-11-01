#!/usr/bin/env node

/**
 * Phase 2: Decide (Keep/Remove)
 * Ranks duplicate files and decides which to keep
 * Rules: test-coverage > import-count > type-definitions > naming-convention
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const projectRoot = process.argv[2] || process.cwd();
const reportPath = process.argv[3]?.split('=')[1] || path.join(projectRoot, '.duplicate-report.json');
const strategy = process.argv[4]?.split('=')[1] || 'test-coverage-first';

const report = JSON.parse(fs.readFileSync(reportPath, 'utf-8'));
const decision = {
  timestamp: new Date().toISOString(),
  strategy,
  keep_remove_map: [],
  scores: {}
};

// Score each file
function scoreFile(filepath) {
  const score = {
    file: filepath,
    testCoverage: 0,
    importCount: 0,
    hasTypeDefinitions: false,
    isConventional: false,
    totalScore: 0
  };

  // Check test coverage
  try {
    const testFile = filepath.replace(/\.(ts|js|tsx|jsx)$/, '.test.$&').replace(/\.(ts|tsx)$/, m => '.spec' + m);
    if (fs.existsSync(testFile)) {
      const testContent = fs.readFileSync(testFile, 'utf-8');
      score.testCoverage = testContent.split('it(').length - 1 + testContent.split('describe(').length - 1;
    }
  } catch (e) {}

  // Count imports
  try {
    const content = fs.readFileSync(filepath, 'utf-8');
    score.importCount = (content.match(/^import|^from/gm) || []).length;
    score.hasTypeDefinitions = /:\s*[A-Z]|interface|type\s+\w+|as\s+const/.test(content);
    score.isConventional = /^src\/(utils|helpers|services|hooks|components)\//.test(filepath);
  } catch (e) {}

  // Calculate total
  score.totalScore = (score.testCoverage * 3) + (score.importCount * 2) + (score.hasTypeDefinitions ? 5 : 0) + (score.isConventional ? 3 : 0);
  
  return score;
}

// Process identical files
if (report.identical && report.identical.length > 0) {
  report.identical.forEach(group => {
    const scores = group.files.map(f => scoreFile(f));
    scores.sort((a, b) => b.totalScore - a.totalScore);
    
    const keep = scores[0].file;
    const remove = scores.slice(1).map(s => s.file);
    
    decision.keep_remove_map.push({
      type: 'identical',
      keep,
      remove,
      reason: `Keeping ${path.basename(keep)} (score: ${scores[0].totalScore})`,
      scores: scores.map(s => ({ file: s.file, score: s.totalScore }))
    });

    remove.forEach(f => {
      decision.scores[f] = { keep, reason: 'identical-duplicate' };
    });
  });
}

// Process duplicate blocks
if (report.duplicates && report.duplicates.length > 0) {
  report.duplicates.forEach(dup => {
    if (dup.percentage >= 80) {
      const scores = dup.files.map(f => scoreFile(f));
      scores.sort((a, b) => b.totalScore - a.totalScore);
      
      const keep = scores[0].file;
      const remove = scores.slice(1).map(s => s.file);
      
      decision.keep_remove_map.push({
        type: 'structural',
        keep,
        remove,
        similarity: dup.percentage,
        lines: dup.lines,
        reason: `${dup.percentage}% similar, keeping higher-scored version`
      });
    }
  });
}

// Process Python duplicates
if (report.python) {
  if (report.python.identical) {
    report.python.identical.forEach(group => {
      const keep = group.files[0];
      const remove = group.files.slice(1);
      
      decision.keep_remove_map.push({
        type: 'python-identical',
        keep,
        remove,
        reason: 'Identical Python modules'
      });
    });
  }
  
  if (report.python.structural) {
    report.python.structural.forEach(group => {
      const keep = group.files[0];
      const remove = group.files.slice(1);
      
      decision.keep_remove_map.push({
        type: 'python-structural',
        keep,
        remove,
        reason: `Same functions/classes: ${group.functions_classes.slice(0, 3).join(', ')}`
      });
    });
  }
}

const outputFile = path.join(projectRoot, '.keep-remove-map.json');
fs.writeFileSync(outputFile, JSON.stringify(decision, null, 2));

console.log(`\n📋 Decision Summary:`);
console.log(`  Strategy: ${strategy}`);
console.log(`  Decisions: ${decision.keep_remove_map.length}`);
console.log(`  Files to remove: ${Object.keys(decision.scores).length}`);
console.log(`  Map: ${outputFile}`);

process.exit(0);
