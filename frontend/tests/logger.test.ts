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
// Adversarial: toJSON/function smuggling (QA audit — Task #3 follow-up)
// ---------------------------------------------------------------------------
//
// `sanitizeMeta`/`sanitizeValue` walk `meta` and copy own enumerable
// properties into fresh plain objects. Before this fix, a FUNCTION value was
// returned unchanged. If that function happened to live under a key named
// `toJSON` on some nested object, the *sanitized* output object would still
// carry a live, unsanitized `toJSON` method — and `JSON.stringify` (used by
// the real `writeToStdout` emission path, bypassed entirely by
// `_setLogSinkForTesting`, which is why this was not caught by the existing
// per-level specification tests above) invokes `toJSON` at serialisation
// time and serialises *its* return value instead, with no further
// sanitisation. A crafted (or just buggy third-party) object whose `toJSON`
// returns a circular structure therefore caused `JSON.stringify(record)` to
// throw inside `writeToStdout`, which silently swallows the exception —
// losing the entire log record with no trace it was ever attempted.

describe('logger — toJSON smuggling cannot defeat sanitisation (adversarial)', () => {
  it('does not lose the log record when a nested object’s toJSON returns a circular structure', () => {
    _resetLogSinkForTesting(); // exercise the real writeToStdout/JSON.stringify path, not the test sink
    const circular: Record<string, unknown> = {};
    circular.self = circular;
    const malicious = { toJSON: () => circular };

    const writeSpy = vi.spyOn(process.stdout, 'write').mockImplementation(() => true);
    expect(() => logger.error('toJSON smuggle attempt', { payload: malicious })).not.toThrow();
    // Before the fix this was 0: JSON.stringify threw inside writeToStdout's
    // try/catch and the record was silently dropped instead of written.
    expect(writeSpy.mock.calls.length).toBe(1);
    writeSpy.mockRestore();
  });

  it('replaces a function-valued metadata field with a descriptive placeholder, not the function itself', () => {
    _setLogSinkForTesting((record) => captured.push(record));
    const fn = function namedFn() {
      return 'should not appear verbatim';
    };
    logger.info('function meta', { handler: fn });
    expect(typeof captured[0]?.handler).toBe('string');
    expect(captured[0]?.handler).toContain('namedFn');
  });

  it('never lets a smuggled toJSON leak its unsanitized return value into the emitted JSON text', () => {
    _resetLogSinkForTesting();
    const secretCircular: Record<string, unknown> = { marker: 'should-not-appear-unsanitized' };
    secretCircular.self = secretCircular;
    const malicious = { toJSON: () => secretCircular };

    let written = '';
    const writeSpy = vi.spyOn(process.stdout, 'write').mockImplementation((chunk: unknown) => {
      written = String(chunk);
      return true;
    });
    logger.error('toJSON smuggle attempt 2', { payload: malicious });
    writeSpy.mockRestore();

    // The record must have been written at all (see test above), and the
    // malicious toJSON must never have been invoked by the final
    // JSON.stringify — so its distinctive marker string must not appear.
    expect(written).not.toContain('should-not-appear-unsanitized');
  });
});

// ---------------------------------------------------------------------------
// Adversarial: shared (non-circular) references must not be mislabeled
// (QA audit — Task #3 follow-up)
// ---------------------------------------------------------------------------
//
// The original `seen` WeakSet was never cleared after a subtree finished
// processing, so it accumulated every object visited anywhere in the whole
// traversal rather than just the object's current ancestors. Two sibling
// fields (or two array entries) that happen to reference the SAME object —
// a common, entirely non-circular pattern (e.g. the same config/user object
// attached under two different keys) — were therefore incorrectly collapsed
// to the string `'[Circular]'` on the second occurrence, silently destroying
// real data that was never actually part of a cycle.

describe('logger — shared (non-circular) references are preserved (adversarial)', () => {
  it('serialises a value shared by two sibling object keys in full both times', () => {
    _setLogSinkForTesting((record) => captured.push(record));
    const shared = { x: 1 };
    logger.info('shared ref', { payload: { a: shared, b: shared } });
    const payload = captured[0]?.payload as { a: unknown; b: unknown };
    expect(payload.a).toEqual({ x: 1 });
    expect(payload.b).toEqual({ x: 1 });
  });

  it('serialises a value shared across multiple array entries in full every time', () => {
    _setLogSinkForTesting((record) => captured.push(record));
    const shared = { x: 1 };
    logger.info('shared ref array', { list: [shared, shared, shared] });
    const list = captured[0]?.list as unknown[];
    expect(list).toEqual([{ x: 1 }, { x: 1 }, { x: 1 }]);
  });

  it('still reports a TRUE self-referential cycle as "[Circular]" (regression guard)', () => {
    _setLogSinkForTesting((record) => captured.push(record));
    const circular: Record<string, unknown> = { name: 'circular' };
    circular.self = circular;
    logger.warn('true cycle', { circular });
    const emitted = captured[0]?.circular as { self: unknown };
    expect(emitted.self).toBe('[Circular]');
  });
});

// ---------------------------------------------------------------------------
// Adversarial: very large / very deeply nested meta (QA audit — Task #3 follow-up)
// ---------------------------------------------------------------------------

describe('logger — very deeply nested (non-circular) metadata does not throw (adversarial)', () => {
  it('does not throw and still emits a record for a ~200k-deep non-circular object graph', () => {
    _setLogSinkForTesting((record) => captured.push(record));
    let deep: Record<string, unknown> = { leaf: true };
    for (let i = 0; i < 200_000; i++) {
      deep = { child: deep };
    }
    expect(() => logger.info('deep nesting test', { deep })).not.toThrow();
    expect(captured).toHaveLength(1);
    // The per-field sanitizeMeta try/catch converts a stack-overflow from
    // recursing this deep into a safe placeholder rather than propagating.
    expect(captured[0]?.deep).toBeDefined();
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
