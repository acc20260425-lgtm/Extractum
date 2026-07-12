import { spawn } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(fileURLToPath(new URL('..', import.meta.url)));

function npmStep(title, scriptName) {
  if (process.env.npm_execpath) {
    return {
      title,
      command: process.execPath,
      args: [process.env.npm_execpath, 'run', scriptName]
    };
  }

  if (process.platform === 'win32') {
    console.error(
      'Unable to locate npm CLI path. Run this command through "npm run verify".'
    );
    process.exit(1);
  }

  return {
    title,
    command: 'npm',
    args: ['run', scriptName]
  };
}

const steps = [
  npmStep('npm run test', 'test'),
  npmStep('npm run check', 'check'),
  npmStep('npm run check:rustfmt', 'check:rustfmt'),
  {
    title: 'cargo check --manifest-path src-tauri/Cargo.toml',
    command: 'cargo',
    args: ['check', '--manifest-path', 'src-tauri/Cargo.toml']
  },
  {
    title: 'cargo test --manifest-path src-tauri/Cargo.toml',
    command: 'cargo',
    args: ['test', '--manifest-path', 'src-tauri/Cargo.toml']
  },
  {
    title: 'git diff HEAD --check',
    command: 'git',
    args: ['diff', 'HEAD', '--check']
  }
];

function runStep(step) {
  return new Promise((resolve) => {
    let settled = false;
    const finish = (exitCode) => {
      if (settled) {
        return;
      }

      settled = true;
      resolve(exitCode);
    };

    console.log(`\n=== ${step.title} ===`);

    const child = spawn(step.command, step.args, {
      cwd: repoRoot,
      shell: false,
      stdio: 'inherit'
    });

    child.on('error', (error) => {
      console.error(`\nFailed to start "${step.command}": ${error.message}`);
      finish(1);
    });

    child.on('close', (code, signal) => {
      if (signal) {
        console.error(`\nCommand terminated by signal ${signal}: ${step.title}`);
        finish(1);
        return;
      }

      finish(code ?? 1);
    });
  });
}

for (const step of steps) {
  const exitCode = await runStep(step);

  if (exitCode !== 0) {
    console.error(`\nVerification failed during: ${step.title}`);
    process.exit(exitCode);
  }
}

console.log('\nAll verification checks passed.');
