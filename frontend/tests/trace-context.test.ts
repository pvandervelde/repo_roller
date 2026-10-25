import { describe, it, expect } from 'vitest';
import fc from 'fast-check';
import {
  TRACEPARENT_FORMAT,
  buildTraceparentHeader,
  getCurrentTraceparent,
  getActiveSpanContext,
  type ActiveSpanContext,
} from '../src/lib/observability/trace-context';

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

const FIXED_PARENT: ActiveSpanContext = {
  traceId: 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
  spanId: 'bbbbbbbbbbbbbbbb',
  sampled: true,
};

/** fast-check arbitrary for a valid 32-hex-char trace id. */
const traceIdArb = fc
  .array(fc.integer({ min: 0, max: 15 }), { minLength: 32, maxLength: 32 })
  .map((digits) => digits.map((d) => d.toString(16)).join(''))
  .filter((id) => !/^0+$/.test(id));

/** fast-check arbitrary for a valid 16-hex-char span id. */
const spanIdArb = fc
  .array(fc.integer({ min: 0, max: 15 }), { minLength: 16, maxLength: 16 })
  .map((digits) => digits.map((d) => d.toString(16)).join(''))
  .filter((id) => !/^0+$/.test(id));

// ---------------------------------------------------------------------------
// Tier 1 — specification tests
// ---------------------------------------------------------------------------

describe('getCurrentTraceparent() — specification', () => {
  it('returns a string matching the W3C traceparent format', () => {
    const header = getCurrentTraceparent();
    expect(header).toMatch(TRACEPARENT_FORMAT);
  });

  it('always uses version byte "00"', () => {
    const header = getCurrentTraceparent();
    expect(header.slice(0, 2)).toBe('00');
  });

  it('never returns a header with an all-zero trace-id', () => {
    const header = getCurrentTraceparent();
    const traceId = header.split('-')[1];
    expect(traceId).not.toBe('0'.repeat(32));
  });

  it('never returns a header with an all-zero parent-id', () => {
    const header = getCurrentTraceparent();
    const spanId = header.split('-')[2];
    expect(spanId).not.toBe('0'.repeat(16));
  });
});

// ---------------------------------------------------------------------------
// Tier 2 — adversarial / stub-killing tests
// ---------------------------------------------------------------------------

// The module under test does not currently expose a way to *inject* an
// active span context from outside (that linkage to the real OTel API is an
// implementation detail for the Coder). We test the two functions that make
// up the public contract directly instead: `buildTraceparentHeader` (pure,
// directly testable continuation/mint behaviour) and `getCurrentTraceparent`
// (the no-active-span fallback path, which is what actually runs in this
// unit test environment since no real OTel SDK is registered).

describe('buildTraceparentHeader() — continuation of an active span', () => {
  it('embeds the exact given trace-id verbatim', () => {
    const header = buildTraceparentHeader(FIXED_PARENT);
    expect(header).toContain(FIXED_PARENT.traceId);
  });

  it('embeds the exact given span-id verbatim', () => {
    const header = buildTraceparentHeader(FIXED_PARENT);
    expect(header).toContain(FIXED_PARENT.spanId);
  });

  it('does not alter the trace-id when called twice with the same parent', () => {
    const first = buildTraceparentHeader(FIXED_PARENT);
    const second = buildTraceparentHeader(FIXED_PARENT);
    expect(first.split('-')[1]).toBe(second.split('-')[1]);
  });

  it('produces a fully well-formed header even when given a parent context', () => {
    const header = buildTraceparentHeader(FIXED_PARENT);
    expect(header).toMatch(TRACEPARENT_FORMAT);
  });
});

describe('buildTraceparentHeader() — fallback mint when there is no parent', () => {
  it('returns a well-formed header when parent is null', () => {
    const header = buildTraceparentHeader(null);
    expect(header).toMatch(TRACEPARENT_FORMAT);
  });

  it('returns a well-formed header when parent is omitted entirely', () => {
    const header = buildTraceparentHeader();
    expect(header).toMatch(TRACEPARENT_FORMAT);
  });

  it('mints a different trace-id on each call with no parent (not a hardcoded constant)', () => {
    const first = buildTraceparentHeader(null);
    const second = buildTraceparentHeader(null);
    expect(first.split('-')[1]).not.toBe(second.split('-')[1]);
  });

  it('mints a different span-id on each call with no parent', () => {
    const first = buildTraceparentHeader(null);
    const second = buildTraceparentHeader(null);
    expect(first.split('-')[2]).not.toBe(second.split('-')[2]);
  });
});

describe('getCurrentTraceparent() — stub-killing uniqueness', () => {
  it('produces a different trace-id across many consecutive calls (no fixed sentinel)', () => {
    const headers = Array.from({ length: 20 }, () => getCurrentTraceparent());
    const traceIds = new Set(headers.map((h) => h.split('-')[1]));
    expect(traceIds.size).toBe(headers.length);
  });

  it('never throws across many consecutive calls', () => {
    for (let i = 0; i < 20; i++) {
      expect(() => getCurrentTraceparent()).not.toThrow();
    }
  });
});

describe('getActiveSpanContext() — specification', () => {
  it('returns either null or an object with traceId/spanId/sampled fields', () => {
    const ctx = getActiveSpanContext();
    if (ctx !== null) {
      expect(ctx).toHaveProperty('traceId');
      expect(ctx).toHaveProperty('spanId');
      expect(ctx).toHaveProperty('sampled');
    } else {
      expect(ctx).toBeNull();
    }
  });
});

// ---------------------------------------------------------------------------
// Tier 3 — property-based tests (fast-check)
// ---------------------------------------------------------------------------

describe('buildTraceparentHeader() — property: well-formed for any valid parent', () => {
  it('always produces a header matching TRACEPARENT_FORMAT and containing the given ids', () => {
    fc.assert(
      fc.property(traceIdArb, spanIdArb, fc.boolean(), (traceId, spanId, sampled) => {
        const header = buildTraceparentHeader({ traceId, spanId, sampled });
        expect(header).toMatch(TRACEPARENT_FORMAT);
        expect(header).toContain(traceId);
        expect(header).toContain(spanId);
      }),
      { numRuns: 100 },
    );
  });

  it('never throws for any syntactically valid parent context', () => {
    fc.assert(
      fc.property(traceIdArb, spanIdArb, fc.boolean(), (traceId, spanId, sampled) => {
        expect(() => buildTraceparentHeader({ traceId, spanId, sampled })).not.toThrow();
      }),
      { numRuns: 100 },
    );
  });
});

describe('getCurrentTraceparent() — property: repeated calls stay well-formed', () => {
  it('every call in a long sequence returns a well-formed, non-empty header', () => {
    fc.assert(
      fc.property(fc.integer({ min: 1, max: 50 }), (n) => {
        for (let i = 0; i < n; i++) {
          const header = getCurrentTraceparent();
          expect(header.length).toBeGreaterThan(0);
          expect(header).toMatch(TRACEPARENT_FORMAT);
        }
      }),
      { numRuns: 20 },
    );
  });
});
