/**
 * Guards the "no remaining raw console.* calls in application code" acceptance
 * criterion using the TypeScript compiler API to find actual CallExpression
 * nodes — not a naive regex, which would either (a) false-positive on the
 * string literal in `src/lib/session.server.ts` that merely *mentions*
 * `console.log` inside an error message intended for a human to read and run
 * in a shell, or (b) false-negative on calls written with unusual whitespace.
 * Parsing the real AST sidesteps both failure modes.
 */
import { describe, it, expect } from 'vitest';
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join, extname } from 'node:path';
import ts from 'typescript';

const SRC_ROOT = join(__dirname, '..', 'src');

/** Directories/files that are allowed to be skipped (none currently — kept explicit). */
const EXCLUDE_DIR_NAMES = new Set(['node_modules']);

function collectSourceFiles(dir: string, acc: string[] = []): string[] {
  for (const entry of readdirSync(dir)) {
    if (EXCLUDE_DIR_NAMES.has(entry)) continue;
    const full = join(dir, entry);
    const stats = statSync(full);
    if (stats.isDirectory()) {
      collectSourceFiles(full, acc);
    } else if (extname(entry) === '.ts' || extname(entry) === '.js') {
      acc.push(full);
    }
  }
  return acc;
}

/** Walk a TS source file's AST and collect every `console.<method>(...)` call's method name. */
function findConsoleCalls(filePath: string): string[] {
  const text = readFileSync(filePath, 'utf-8');
  const sourceFile = ts.createSourceFile(
    filePath,
    text,
    ts.ScriptTarget.Latest,
    /* setParentNodes */ true,
    filePath.endsWith('.ts') ? ts.ScriptKind.TS : ts.ScriptKind.JS,
  );

  const found: string[] = [];

  function visit(node: ts.Node) {
    if (
      ts.isCallExpression(node) &&
      ts.isPropertyAccessExpression(node.expression) &&
      ts.isIdentifier(node.expression.expression) &&
      node.expression.expression.text === 'console'
    ) {
      found.push(node.expression.name.text);
    }
    ts.forEachChild(node, visit);
  }

  visit(sourceFile);
  return found;
}

describe('no raw console.* calls in application source (AST-based)', () => {
  const files = collectSourceFiles(SRC_ROOT);

  it('scans at least one source file (sanity check the scanner itself runs)', () => {
    expect(files.length).toBeGreaterThan(0);
  });

  it('finds zero console.* CallExpression nodes across all of src/**/*.{ts,js}', () => {
    const offenders: Array<{ file: string; calls: string[] }> = [];
    for (const file of files) {
      const calls = findConsoleCalls(file);
      if (calls.length > 0) {
        offenders.push({ file, calls });
      }
    }
    expect(offenders).toEqual([]);
  });

  it('does not false-positive on the console.log mention inside session.server.ts error string', () => {
    const file = join(SRC_ROOT, 'lib', 'session.server.ts');
    // Confirms the known tricky case specifically: this file's AST must
    // report zero console.* calls even though the raw text contains the
    // substring "console.log(" inside a string literal.
    expect(findConsoleCalls(file)).toEqual([]);
  });
});
