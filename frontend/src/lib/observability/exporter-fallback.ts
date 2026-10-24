/**
 * Shared "fail-safe-off" span exporter selection logic, factored out of
 * `observability.server.ts`'s `buildServerSpanExporter()` and
 * `observability.client.ts`'s `buildClientSpanExporter()` — both followed
 * the identical shape (gate on an enabled/configured flag, try to construct
 * the real OTLP exporter, fall back to a no-op exporter on either path)
 * despite using different OTel SDK packages (`sdk-trace-node` vs.
 * `sdk-trace-web`) and different OTLP transports (gRPC vs. HTTP) for the
 * "real" exporter itself.
 *
 * Deliberately generic over the exporter type `E`: the caller supplies both
 * the "real" and "fallback" constructors as closures, so this module never
 * needs to import any OTel SDK package itself (and therefore never needs to
 * pick a side between the two SDKs' `ConsoleSpanExporter` instances, which
 * each caller already imports from whichever package it uses).
 */

/**
 * Select and construct a `SpanExporter`, fail-safe-off:
 *
 * - When `shouldExportReal` is `false`, returns `buildFallback()` directly —
 *   the real exporter is never constructed, so no network configuration is
 *   even touched when nobody asked for export.
 * - When `shouldExportReal` is `true`, returns `buildReal()`'s result. If
 *   `buildReal()` throws (e.g. a malformed endpoint, if the underlying
 *   exporter package validates eagerly), calls `onFallback` (if given) and
 *   returns `buildFallback()` instead — exporter-construction failure must
 *   never propagate and break server startup or the page.
 *
 * MUST NOT throw: `buildFallback()` itself is assumed infallible (the
 * no-op/console exporter it constructs performs no I/O), matching both
 * current call sites.
 */
export function buildFailSafeSpanExporter<E>(
  shouldExportReal: boolean,
  buildReal: () => E,
  buildFallback: () => E,
  onFallback?: (err: unknown) => void,
): E {
  if (!shouldExportReal) {
    return buildFallback();
  }

  try {
    return buildReal();
  } catch (err) {
    onFallback?.(err);
    return buildFallback();
  }
}
