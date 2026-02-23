/**
 * RepoRoller Webhook Receiver — Node.js Example
 * ===============================================
 * Demonstrates how to receive and verify RepoRoller outbound webhook notifications.
 *
 * Requirements:
 *   node >= 18  (built-in `crypto` module; no extra dependencies)
 *
 * Usage:
 *   WEBHOOK_SECRET="your-shared-secret-value" node receiver.js
 *
 * The server listens on port 8080 and accepts POST /webhook requests.
 *
 * See docs/notifications.md for full webhook documentation.
 */

'use strict';

const crypto = require('crypto');
const http = require('http');

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

const WEBHOOK_SECRET = process.env.WEBHOOK_SECRET;
if (!WEBHOOK_SECRET) {
    console.error('ERROR: WEBHOOK_SECRET environment variable is not set');
    process.exit(1);
}

const PORT = parseInt(process.env.PORT || '8080', 10);

// ---------------------------------------------------------------------------
// Signature verification
// ---------------------------------------------------------------------------

/**
 * Returns true if the given signature header matches the HMAC-SHA256 of the body.
 *
 * Uses `crypto.timingSafeEqual` to prevent timing attacks.
 *
 * @param {Buffer} rawBody - The raw, unparsed request body bytes.
 * @param {string} signatureHeader - Value of `X-RepoRoller-Signature-256` header.
 *                                    Expected format: `sha256=<hex>`.
 * @returns {boolean}
 */
function verifySignature(rawBody, signatureHeader) {
    if (!signatureHeader || !signatureHeader.startsWith('sha256=')) {
        return false;
    }

    const receivedHex = signatureHeader.slice('sha256='.length);

    const hmac = crypto.createHmac('sha256', WEBHOOK_SECRET);
    hmac.update(rawBody);
    const computedHex = hmac.digest('hex');

    // Constant-time comparison — never use === here.
    const receivedBuf = Buffer.from(receivedHex, 'hex');
    const computedBuf = Buffer.from(computedHex, 'hex');

    if (receivedBuf.length !== computedBuf.length) {
        return false;
    }

    return crypto.timingSafeEqual(receivedBuf, computedBuf);
}

// ---------------------------------------------------------------------------
// Event handlers
// ---------------------------------------------------------------------------

/**
 * Handle a `repository.created` event.
 * @param {object} payload - Parsed event payload.
 */
function handleRepositoryCreated(payload) {
    const {
        event_id,
        organization,
        repository_name,
        repository_url,
        created_by,
        visibility,
        template_name,
        team,
        content_strategy,
    } = payload;

    console.info(
        `Repository created — event_id=${event_id} org=${organization} ` +
        `name=${repository_name} url=${repository_url} created_by=${created_by} ` +
        `visibility=${visibility} template=${template_name ?? '(none)'} ` +
        `team=${team ?? '(none)'} strategy=${content_strategy}`
    );

    // Add your integration logic here:
    //   - Post a Slack / Teams notification
    //   - Register the repo in a service catalog
    //   - Trigger a CI provisioning pipeline
    //   - Update an asset inventory database
}

// ---------------------------------------------------------------------------
// HTTP server
// ---------------------------------------------------------------------------

const server = http.createServer((req, res) => {
    const url = new URL(req.url, `http://${req.headers.host}`);

    if (req.method !== 'POST' || url.pathname !== '/webhook') {
        res.writeHead(404);
        res.end('Not Found');
        return;
    }

    // Collect raw body bytes BEFORE parsing — signature covers the raw body.
    const chunks = [];
    req.on('data', (chunk) => chunks.push(chunk));
    req.on('end', () => {
        const rawBody = Buffer.concat(chunks);
        const signatureHeader = req.headers['x-reporoller-signature-256'] || '';

        // 1. Verify signature.
        if (!verifySignature(rawBody, signatureHeader)) {
            console.warn(`Rejected request with invalid signature from ${req.socket.remoteAddress}`);
            res.writeHead(401);
            res.end('Unauthorized');
            return;
        }

        // 2. Parse payload.
        let payload;
        try {
            payload = JSON.parse(rawBody.toString('utf8'));
        } catch (err) {
            console.error(`Failed to parse JSON body: ${err.message}`);
            res.writeHead(400);
            res.end('Bad Request');
            return;
        }

        // 3. Dispatch on event type.
        const eventType = payload.event_type;
        if (eventType === 'repository.created') {
            handleRepositoryCreated(payload);
        } else {
            console.info(`Ignoring unknown event type: ${eventType}`);
        }

        // Always acknowledge promptly — processing is fire-and-forget from sender's perspective.
        res.writeHead(204);
        res.end();
    });

    req.on('error', (err) => {
        console.error(`Request error: ${err.message}`);
        res.writeHead(500);
        res.end('Internal Server Error');
    });
});

server.listen(PORT, '0.0.0.0', () => {
    console.info(`RepoRoller webhook receiver listening on port ${PORT}`);
    // In production, terminate TLS at a reverse proxy / load balancer.
    // Your notifications.toml endpoint URL must use https://.
});
