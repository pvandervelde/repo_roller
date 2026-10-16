import { initClientObservability } from '$lib/observability/observability.client';

// Bootstrap the browser OTel SDK exactly once when this module is first
// loaded by the client (before the app mounts).
initClientObservability();
