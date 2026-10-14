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

function notImplemented(level: string): never {
  throw new Error(`logger.server.ts: logger.${level}() not implemented`);
}

/**
 * The shared server-side structured logger instance. Import this directly;
 * do not construct your own.
 *
 * STUB: every method throws until implemented.
 */
export const logger: Logger = {
  debug: () => notImplemented('debug'),
  info: () => notImplemented('info'),
  warn: () => notImplemented('warn'),
  error: () => notImplemented('error'),
};

/**
 * Test-only hook: redirect log output to a custom sink instead of the real
 * backend (e.g. pino writing to stdout), so tests can assert on the
 * structured record produced by a log call without depending on stdout
 * capture or on which specific logging library backs this module.
 *
 * Must be implemented alongside `logger` itself — currently unused by the
 * stub (there is no real emission pathway yet to redirect).
 */
export function _setLogSinkForTesting(_sink: LogSink): void {
  throw new Error('logger.server.ts: _setLogSinkForTesting() not implemented');
}

/** Test-only hook: remove any sink set via `_setLogSinkForTesting` and restore normal output. */
export function _resetLogSinkForTesting(): void {
  // Intentionally a no-op for now: there is no sink-override state to reset
  // yet because the real implementation does not exist yet.
}
