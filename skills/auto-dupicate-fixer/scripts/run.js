#!/usr/bin/env node

/**
 * Main Runner: Execute full pipeline
 * Orchestrates all 5 phases
 */

const { execSync } = require('child_process');
const path = require('path');
const fs = require('fs');

const projectRoot = process.argv[2] || process.cwd();
const mode = process.argv[3] || 'daily';
const dryRun = process.argv.includes('--dry-run');

const scriptsDir = path.dirname(__filename);
const runLog = {
  timestamp: new Date().toISOString(),
  mode,
  dryRun,
  phases: {}
};

console.log(`🚀 Auto Duplicate Fixer Pipeline`);
console.log(`📍 Project: ${projectRoot}`);
console.log(`⚙️  Mode: ${mode}`);
console.log(`🔒 Dry Run: ${dryRun ? 'YES' : 'NO'}\n`);

const runPhase = (name, command) => {
  console.log(`\n${'='.repeat(50)}`);
  console.log(`Phase: ${name}`);
  console.log(`${'='.repeat(50)}`);
  
  try {
    execSync(command, { 
      cwd: projectRoot, 
      stdio: 'inherit',
      env: { ...process.env, PROJECT_ROOT: projectRoot }
    });
    runLog.phases[name] = { status: 'success' };
    console.log(`✅ ${name} complete\n`);
  } catch (e) {
    runLog.phases[name] = { status: 'failed', error: e.message };
    console.error(`❌ ${name} failed\n`);
    throw e;
  }
};

try {
  // Phase 1: Detect
  runPhase(
    'Detect',
    `node ${scriptsDir}/detect.js ${projectRoot} --format=json --min-tokens=50`
  );

  // Phase 2: Decide
  runPhase(
    'Decide',
    `node ${scriptsDir}/decide.js ${projectRoot} --report=${path.join(projectRoot, '.duplicate-report.json')} --strategy=test-coverage-first`
  );

  if (!dryRun) {
    // Phase 3: Refactor
    runPhase(
      'Refactor',
      `node ${scriptsDir}/refactor.js ${projectRoot}`
    );

    // Phase 4: Validate
    runPhase(
      'Validate',
      `bash ${scriptsDir}/validate.sh ${projectRoot}`
    );

    // Phase 5: Deploy
    runPhase(
      'Deploy',
      `node ${scriptsDir}/deploy.js ${projectRoot} --auto-merge`
    );
  } else {
    console.log(`\n🔍 DRY RUN MODE: Skipping refactor, validate, deploy`);
    console.log(`Report available at: ${path.join(projectRoot, '.keep-remove-map.json')}`);
  }

  runLog.status = 'completed';
  console.log(`\n${'='.repeat(50)}`);
  console.log(`✅ Pipeline Completed Successfully!`);
  console.log(`${'='.repeat(50)}\n`);

} catch (e) {
  runLog.status = 'failed';
  console.error(`\n${'='.repeat(50)}`);
  console.error(`❌ Pipeline Failed`);
  console.error(`${'='.repeat(50)}\n`);
  
  if (!dryRun) {
    console.log('🔙 Rolling back changes...');
    try {
      execSync('git checkout -- .', { cwd: projectRoot });
      console.log('✅ Rollback complete');
    } catch (rollbackError) {
      console.error('⚠️ Rollback failed:', rollbackError.message);
    }
  }
}

const logFile = path.join(projectRoot, '.pipeline-log.json');
fs.writeFileSync(logFile, JSON.stringify(runLog, null, 2));

console.log(`📊 Logs saved: ${logFile}\n`);
process.exit(runLog.status === 'completed' ? 0 : 1);
