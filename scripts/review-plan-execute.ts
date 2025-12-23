#!/usr/bin/env bun

/**
 * Review → Plan → Execute workflow
 *
 * Reads a review markdown file from STDIN, then:
 * 1. Calls `/plan {{FILE}}` to create an execution plan
 * 2. Extracts the plan filename from output
 * 3. Calls `/execute-plan {{FILE}}` to execute the plan
 *
 * Usage:
 *   echo "path/to/review.md" | bun scripts/review-plan-execute.ts [scope]
 *
 * Scope options (case-insensitive):
 *   - "must fix" - Fix only critical issues
 *   - "suggested improvements" - Fix only suggested improvements
 *   - "all suggestions" - Fix all issues (default)
 */

import Bun from "bun";
import { resolve } from 'path';
import { processClaudeStream } from '/Users/ken/.claude/scripts/processClaudeStream';

const CLAUDE_CMD = 'claude';
const CLAUDE_ARGS_STREAM = [
  '-p',
  '--dangerously-skip-permissions',
  '--output-format', 'stream-json',
  '--include-partial-messages',
  '--verbose'
];

/**
 * Execute a Claude Code slash command with real-time output
 */
async function executeClaudeCommand(command: string): Promise<void> {
  console.error(`\n${'='.repeat(80)}`);
  console.error(`Executing: ${command}`);
  console.error(`${'='.repeat(80)}\n`);

  // Use streaming mode and parse JSON to display text
  const proc = Bun.spawn([CLAUDE_CMD, ...CLAUDE_ARGS_STREAM, command], {
    stdout: 'pipe',
    stderr: 'inherit',
    stdin: 'inherit',
  });

  await processClaudeStream(proc.stdout);

  const exitCode = await proc.exited;

  if (exitCode !== 0) {
    throw new Error(`Command failed with exit code ${exitCode}`);
  }
}

/**
 * Extract L2 headings (##) from markdown content
 */
function extractL2Headings(markdown: string): string[] {
  const headings: string[] = [];
  const lines = markdown.split('\n');

  for (const line of lines) {
    const match = line.match(/^##\s+(.+)$/);
    if (match && match[1]) {
      headings.push(match[1].trim());
    }
  }

  return headings;
}

/**
 * Capitalize first letter of each word
 */
function capitalizeWords(text: string): string {
  return text
    .toLowerCase()
    .split(' ')
    .map(word => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ');
}

/**
 * Prompt user for confirmation
 */
async function confirm(message: string): Promise<boolean> {
  process.stderr.write(`\n${message} (y/n): `);

  // Read from stdin
  const n = await Bun.stdin.stream().getReader().read();

  if (n.value) {
    const answer = new TextDecoder().decode(n.value).trim().toLowerCase();
    return answer === 'y' || answer === 'yes';
  }

  return false;
}

/**
 * Find the most recently modified plan file in .ai/plans/
 */
async function findNewestPlanFile(): Promise<string | null> {
  const plansDir = '.ai/plans';

  try {
    const files = await Array.fromAsync(
      new Bun.Glob('*.md').scan({ cwd: plansDir })
    );

    if (files.length === 0) {
      return null;
    }

    // Get file stats and sort by modification time
    const filesWithStats = await Promise.all(
      files.map(async (file) => {
        const fullPath = `${plansDir}/${file}`;
        const stat = await Bun.file(fullPath).stat();
        return { path: fullPath, mtime: stat.mtime };
      })
    );

    filesWithStats.sort((a, b) => b.mtime.getTime() - a.mtime.getTime());

    return filesWithStats && filesWithStats[0] ? filesWithStats[0].path : null;
  } catch (error) {
    console.error('Error finding plan file:', error);
    return null;
  }
}

/**
 * Main workflow
 */
async function main() {
  try {
    // Parse command line args: bun script.ts [scope]
    const args = Bun.argv.slice(2); // Remove 'bun' and script path
    let scopeArg = args[0];

    // Read review filename from STDIN
    const stdin = await Bun.stdin.text();
    const reviewFile = stdin.trim();

    if (!reviewFile) {
      console.error('Error: No review file provided via STDIN');
      console.error('Usage: echo "path/to/review.md" | bun scripts/review-plan-execute.ts [scope]');
      console.error('\nScope options: "must fix", "suggested improvements", "all suggestions" (default)');
      process.exit(1);
    }

    const reviewPath = resolve(reviewFile);
    console.error(`Review file: ${reviewPath}\n`);

    // Read review content to extract available sections
    const reviewContent = await Bun.file(reviewPath).text();
    const headings = extractL2Headings(reviewContent);

    // Determine scope
    let scope: string;

    if (!scopeArg) {
      // Default to "all suggestions" with confirmation
      console.error('Available sections in review:');
      headings.forEach((h, i) => console.error(`  ${i + 1}. ${h}`));
      console.error('\nScope options:');
      console.error('  - "must fix" - Address only critical issues');
      console.error('  - "suggested improvements" - Address only suggestions');
      console.error('  - "all suggestions" - Address all issues (default)');

      const confirmed = await confirm('Proceed with "All Suggestions" scope?');

      if (!confirmed) {
        console.error('\nAborted by user');
        process.exit(0);
      }

      scope = 'All Suggestions';
    } else {
      // Capitalize user input
      scope = capitalizeWords(scopeArg);
    }

    console.error(`\nScope: ${scope}\n`);

    // Step 1: Create plan using /plan command with directive
    const planPrompt = `/plan Fix "${scope}" issues from ${reviewPath}`;
    await executeClaudeCommand(planPrompt);

    // Step 2: Find the newest plan file
    const planFile = await findNewestPlanFile();

    if (!planFile) {
      console.error('\n❌ Failed to find plan file in .ai/plans/');
      process.exit(1);
    }

    console.error(`\n✓ Plan created: ${planFile}\n`);

    // Step 3: Execute plan
    await executeClaudeCommand(`/execute-plan ${planFile}`);

    console.error('\n✓ Plan execution complete\n');

  } catch (error) {
    console.error('\n❌ Error:', error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}

main();
