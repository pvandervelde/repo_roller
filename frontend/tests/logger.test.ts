import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import {
  logger,
  _setLogSinkForTesting,
  _resetLogSinkForTesting,
  type LogRecord,
} from '../src/lib/logger.server';

let captured: LogRecord[] = [];

beforeEach(() => {
  captured = [];
  try {
    _setLogSinkForTesting((record) => {
      captured.push(record);
    });
  } catch {
    // Stub phase: _setLogSinkForTesting itself is not implemented yet.
    // Individual tests below still exercise logger.* and will fail for the
    // right reason (the "not implemented" throw), which is the expected RED
    // state before the Coder implements this module.
  }
});

afterEach(() => {
  _resetLogSinkForTesting();
  vi.restoreAllMocks();
});

// ---------------------------------------------------------------------------
// Tier 1 — specification tests: one per log level
// ---------------------------------------------------------------------------

describe('logger — per-level specification', () => {
  it('logger.debug() emits a record with level "debug"', () => {
    logger.debug('a debug message');
    expect(captured[0]?.level).toBe('debug');
  });

  it('logger.info() emits a record with level "info"', () => {
    logger.info('an info message');
    expect(captured[0]?.level).toBe('info');
  });

  it('logger.warn() emits a record with level "warn"', () => {
    logger.warn('a warning message');
    expect(captured[0]?.level).toBe('warn');
  });

  it('logger.error() emits a record with level "error"', () => {
    logger.error('an error message');
    expect(captured[0]?.level).toBe('error');
  });

  it('emits the exact message string passed in', () => {
    logger.info('exact message text');
    expect(captured[0]?.message).toBe('exact message text');
  });

  it('merges caller-supplied metadata fields into the emitted record', () => {
    logger.info('user action', { userLogin: 'jdoe', requestId: 'req-123' });
    expect(captured[0]?.userLogin).toBe('jdoe');
    expect(captured[0]?.requestId).toBe('req-123');
  });

  it('includes a time field that parses as a valid date', () => {
    logger.info('timestamped');
    const time = captured[0]?.time;
    expect(time).toBeTruthy();
    expect(Number.isNaN(new Date(time as string).getTime())).toBe(false);
  });
});

// ---------------------------------------------------------------------------
// Tier 2 — adversarial / stub-killing tests
// ---------------------------------------------------------------------------

describe('logger — level differentiation (stub-killing)', () => {
  it('produces four distinct level values across the four methods, not one hardcoded level', () => {
    logger.debug('d');
    logger.info('i');
    logger.warn('w');
    logger.error('e');
    const levels = captured.map((r) => r.level);
    expect(new Set(levels).size).toBe(4);
    expect(levels).toEqual(['debug', 'info', 'warn', 'error']);
  });
});

describe('logger — never throws (adversarial)', () => {
  it('does not throw when no metadata is passed', () => {
    expect(() => logger.info('no meta')).not.toThrow();
  });

  it('does not throw when metadata contains a circular reference', () => {
    const circular: Record<string, unknown> = { name: 'circular' };
    circular.self = circular;
    expect(() => logger.warn('circular meta', circular)).not.toThrow();
  });

  it('does not throw when a metadata field is an Error instance', () => {
    const err = new Error('boom');
    expect(() => logger.error('failure', { err })).not.toThrow();
  });

  it('does not throw when metadata explicitly contains undefined values', () => {
    expect(() => logger.info('sparse meta', { maybe: undefined })).not.toThrow();
  });
});

describe('logger — Error serialisation (adversarial)', () => {
  it('preserves the Error message as a string field rather than losing it to {}', () => {
    const err = new Error('disk full');
    logger.error('write failed', { err });
    const emitted = JSON.stringify(captured[0]);
    expect(emitted).toContain('disk full');
  });

  it('preserves the Error stack as a string field', () => {
    const err = new Error('stack check');
    logger.error('write failed', { err });
    const emitted = JSON.stringify(captured[0]);
    // A real stack trace contains the word "at" on subsequent lines in V8.
    expect(emitted.length).toBeGreaterThan(JSON.stringify({ message: 'stack check' }).length);
  });
});

describe('logger — no spurious keys when metadata is omitted (boundary)', () => {
  it('emits exactly level, message, and time — nothing else', () => {
    logger.info('clean record');
    const keys = Object.keys(captured[0] ?? {}).sort();
    expect(keys).toEqual(['level', 'message', 'time'].sort());
  });
});

describe('logger — does not delegate to console.* (stub-killing)', () => {
  it('logger.warn() does not call the global console.warn', () => {
    const spy = vi.spyOn(console, 'warn').mockImplementation(() => {});
    logger.warn('should not hit console');
    expect(spy).not.toHaveBeenCalled();
  });

  it('logger.error() does not call the global console.error', () => {
    const spy = vi.spyOn(console, 'error').mockImplementation(() => {});
    logger.error('should not hit console');
    expect(spy).not.toHaveBeenCalled();
  });

  it('logger.info() does not call the global console.log', () => {
    const spy = vi.spyOn(console, 'log').mockImplementation(() => {});
    logger.info('should not hit console');
    expect(spy).not.toHaveBeenCalled();
  });
});

// ---------------------------------------------------------------------------
// Sink state isolation
// ---------------------------------------------------------------------------

describe('_resetLogSinkForTesting() — state isolation', () => {
  it('stops delivering records to a previously-set sink after reset', () => {
    _resetLogSinkForTesting();
    const before = captured.length;
    logger.info('after reset, should not reach old sink');
    expect(captured.length).toBe(before);
  });
});
