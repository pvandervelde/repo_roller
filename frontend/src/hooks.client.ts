import { env } from '$env/dynamic/public';
import { initClientObservability } from '$lib/observability/observability.client';

// Bootstrap the browser OTel SDK exactly once when this module is first
// loaded by the client (before the app mounts). The OTLP endpoint must be
// read via `$env/dynamic/public` (runtime config, browser-exposed) rather
// than resolved internally by `observability.client.ts` — see that module's
// doc comment ("Real OTLP exporter wiring") for why the browser cannot
// resolve this value itself.
initClientObservability({ otlpEndpoint: env.PUBLIC_OTEL_EXPORTER_OTLP_ENDPOINT });
