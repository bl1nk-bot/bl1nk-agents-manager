#!/usr/bin/env node

/**
 * Phase 5: Deploy
 * Creates PR or commits changes
 * Handles auto-merge if CI passes
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const projectRoot = process.argv[2] || process.cwd();
const autoMerge = process.argv[3]?.includes('--auto-merge');
const title = "Auto-fix: Remove duplicates and refactor imports";

const deployLog = {
  timestamp: new Date().toISOString(),
  pr_created: false,
  pr_url: null,
  committed: false,
  errors: []
};

try {
  // Check if there are changes
  const status = execSync('git status --porcelain', { encoding: 'utf-8', cwd: projectRoot });
  
  if (!status.trim()) {
    console.log('✅ No changes to commit');
    deployLog.committed = true;
    fs.writeFileSync(path.join(projectRoot, '.deploy-log.json'), JSON.stringify(deployLog, null, 2));
    process.exit(0);
  }

  // Commit changes
  console.log('📝 Committing changes...');
  execSync('git add -A', { cwd: projectRoot });
  
  const refactorLog = JSON.parse(
    fs.readFileSync(path.join(projectRoot, '.refactor-log.json'), 'utf-8')
  );
  
  const commitMessage = `${title}

Files deleted: ${refactorLog.deleted_files.length}
Imports updated: ${refactorLog.updated_imports.length}

Automated by auto-duplicate-fixer skill`;

  execSync(`git commit -m "${commitMessage}"`, { cwd: projectRoot });
  deployLog.committed = true;
  console.log('✅ Committed');

  // Try to create PR (if on CI like GitHub Actions)
  try {
    if (process.env.GITHUB_TOKEN && process.env.GITHUB_REPOSITORY) {
      console.log('📮 Creating PR...');
      
      const branch = `auto-fix/duplicates-${Date.now()}`;
      execSync(`git checkout -b ${branch}`, { cwd: projectRoot });
      execSync('git push origin ' + branch, { cwd: projectRoot });
      
      // Use GitHub API
      const [owner, repo] = process.env.GITHUB_REPOSITORY.split('/');
      const prResponse = execSync(
        `curl -X POST https://api.github.com/repos/${owner}/${repo}/pulls ` +
        `-H "Authorization: token ${process.env.GITHUB_TOKEN}" ` +
        `-d '{"title":"${title}","body":"Automated duplicate file removal and refactoring","head":"${branch}","base":"main"}'`,
        { encoding: 'utf-8' }
      );
      
      const prData = JSON.parse(prResponse);
      deployLog.pr_created = true;
      deployLog.pr_url = prData.html_url;
      console.log(`✅ PR created: ${prData.html_url}`);
    }
  } catch (e) {
    console.log('⚠️ Could not create PR (not in CI environment)');
  }

} catch (e) {
  deployLog.errors.push(e.message);
  console.error('❌ Deploy failed:', e.message);
}

const outputFile = path.join(projectRoot, '.deploy-log.json');
fs.writeFileSync(outputFile, JSON.stringify(deployLog, null, 2));

console.log(`\n🚀 Deploy Summary:`);
console.log(`  Committed: ${deployLog.committed ? '✅' : '❌'}`);
console.log(`  PR Created: ${deployLog.pr_created ? '✅' : '❌'}`);
if (deployLog.pr_url) console.log(`  PR URL: ${deployLog.pr_url}`);
console.log(`  Log: ${outputFile}`);

process.exit(deployLog.errors.length > 0 ? 1 : 0);
