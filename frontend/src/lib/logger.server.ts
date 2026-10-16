/**
 * Server-only structured logger.
 *
 * STATUS: STUB — not yet implemented. Every level function currently
 * throws. This file defines the CONTRACT the Coder must implement; it is
 * not production code.
 *
 * MUST NOT be imported from client-side code (server-only: the real
 * implementation may use `process.env` / `process.stdout`, etc).
 *
 * Scoping decision (Tester, RED phase): this logger is intentionally
 * SERVER-ONLY. The only genuine `console.*` call site found by audit
 * (`src/lib/brand.server.ts:66`, a `console.warn`) is itself server-only
 * code, and the task's OTel instrumentation requirement covers browser-side
 * observability separately (tracing, not logging) — so there is currently
 * no requirement for a browser-safe structured logger. Scoping to
 * server-only keeps this module free to depend on Node-only structured
 * logging libraries (the task brief suggests `pino`) without needing to
 * worry about browser bundle size or browser API availability.
 *
 * Contract for the Coder:
 * - MUST implement all four levels below: debug, info, warn, error.
 * - MUST emit a STRUCTURED record (an object with at least `level`,
 *   `message`, and `time` fields, plus any caller-supplied metadata fields
 *   merged in) — not a plain formatted string passed to `console.*`. In
 *   particular, `logger.warn()` and `logger.error()` MUST NOT call the
 *   global `console.warn`/`console.error` as their mechanism of emission
 *   (that would just be relocating the problem this logger exists to fix).
 * - The `level` field of the emitted record MUST match the function that
 *   was called (`logger.debug()` → `level: 'debug'`, etc.) — a single
 *   level hard-coded for every call is not a conforming implementation.
 * - `time` MUST be a valid ISO-8601 timestamp string, generated at the
 *   moment of the call.
 * - MUST NEVER throw, regardless of what `meta` contains — including when
 *   `meta` has a circular reference, when `meta` is `undefined`, or when a
 *   field under `meta` is itself an `Error` instance. An `Error` value under
 *   `meta` MUST be serialised with at least its `message` and `stack`
 *   preserved as strings (plain `JSON.stringify(someError)` silently
 *   produces `{}` and must specifically be avoided for this case).
 * - When no `meta` is passed, the emitted record MUST NOT contain a
 *   spurious `meta: undefined` (or similar) key — it should contain exactly
 *   `level`, `message`, `time` and nothing else.
 */

export interface LogMetadata {
  [key: string]: unknown;
}

/** The structured record format logger output MUST take (see `_setLogSinkForTesting`). */
export interface LogRecord extends LogMetadata {
  level: 'debug' | 'info' | 'warn' | 'error';
  message: string;
  /** ISO-8601 timestamp, generated at call time. */
  time: string;
}

export interface Logger {
  debug(message: string, meta?: LogMetadata): void;
  info(message: string, meta?: LogMetadata): void;
  warn(message: string, meta?: LogMetadata): void;
  error(message: string, meta?: LogMetadata): void;
}

export type LogSink = (record: LogRecord) => void;

/** Overridden by `_setLogSinkForTesting`; `null` means "use the real default sink". */
let testSink: LogSink | null = null;

/**
 * Recursively convert `value` into something that is always safely
 * JSON-serialisable:
 *  - `Error` instances become a plain object preserving `name`/`message`/
 *    `stack` as strings (a bare `JSON.stringify(someError)` silently
 *    produces `{}`, which this specifically avoids).
 *  - Circular object references are replaced with the string `'[Circular]'`
 *    instead of recursing forever / throwing.
 *  - Everything else is returned unchanged.
 */
function sanitizeValue(value: unknown, seen: WeakSet<object>): unknown {
  if (value instanceof Error) {
    return { name: value.name, message: value.message, stack: value.stack };
  }
  if (value !== null && typeof value === 'object') {
    if (seen.has(value)) return '[Circular]';
    seen.add(value);
    if (Array.isArray(value)) {
      return value.map((item) => sanitizeValue(item, seen));
    }
    const out: Record<string, unknown> = {};
    for (const [key, nested] of Object.entries(value)) {
      out[key] = sanitizeValue(nested, seen);
    }
    return out;
  }
  return value;
}

/** Sanitize every field of `meta` so the resulting object can never throw when serialised. */
function sanitizeMeta(meta: LogMetadata | undefined): LogMetadata {
  if (!meta) return {};
  const result: LogMetadata = {};
  for (const [key, value] of Object.entries(meta)) {
    try {
      result[key] = sanitizeValue(value, new WeakSet<object>());
    } catch {
      result[key] = '[Unserializable]';
    }
  }
  return result;
}

/**
 * Real (non-test) emission path: write one structured JSON line to stdout.
 * Never throws — logging must never be able to break the caller.
 */
function writeToStdout(record: LogRecord): void {
  try {
    process.stdout.write(`${JSON.stringify(record)}\n`);
  } catch {
    // Swallow: a logging failure (e.g. a closed stdout stream) must never
    // propagate to the caller.
  }
}

function emit(level: LogRecord['level'], message: string, meta?: LogMetadata): void {
  const record: LogRecord = {
    ...sanitizeMeta(meta),
    level,
    message,
    time: new Date().toISOString(),
  };

  if (testSink) {
    testSink(record);
    return;
  }

  writeToStdout(record);
}

/**
 * The shared server-side structured logger instance. Import this directly;
 * do not construct your own.
 */
export const logger: Logger = {
  debug: (message, meta) => emit('debug', message, meta),
  info: (message, meta) => emit('info', message, meta),
  warn: (message, meta) => emit('warn', message, meta),
  error: (message, meta) => emit('error', message, meta),
};

/**
 * Test-only hook: redirect log output to a custom sink instead of the real
 * backend (writing structured JSON to stdout), so tests can assert on the
 * structured record produced by a log call without depending on stdout
 * capture or on which specific logging library backs this module.
 */
export function _setLogSinkForTesting(sink: LogSink): void {
  testSink = sink;
}

/** Test-only hook: remove any sink set via `_setLogSinkForTesting` and restore normal output. */
export function _resetLogSinkForTesting(): void {
  testSink = null;
}
