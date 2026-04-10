/**
 * Server-only session cookie signing and verification.
 *
 * Uses HMAC-SHA256 with SESSION_SECRET to sign session payloads, preventing
 * cookie forgery. The cookie value has the format:
 *
 *   <base64url(payload)>.<hex(hmac-sha256(base64url(payload), SESSION_SECRET))>
 *
 * SESSION_SECRET must be at least 32 characters. Rotate it to invalidate all
 * existing sessions.
 *
 * MUST NOT be imported from client-side code.
 */
import { createHmac, timingSafeEqual } from 'node:crypto';

const HMAC_ALGO = 'sha256';
const MIN_SECRET_LEN = 32;
/** HMAC-SHA256 always produces a 32-byte (64-hex-char) digest. */
const DIGEST_HEX_LEN = 64;

function getSecret(): string {
    const secret = process.env.SESSION_SECRET;
    if (!secret || secret.length < MIN_SECRET_LEN) {
        throw new Error(
            `SESSION_SECRET environment variable must be set to at least ${MIN_SECRET_LEN} characters. ` +
            'Generate one with: node -e "console.log(require(\'crypto\').randomBytes(48).toString(\'hex\'))"',
        );
    }
    return secret;
}

/**
 * Sign a session payload and return the cookie string value.
 *
 * @throws if SESSION_SECRET is not set or too short.
 */
export function signSessionCookie(payload: object): string {
    const secret = getSecret();
    const data = Buffer.from(JSON.stringify(payload)).toString('base64url');
    const sig = createHmac(HMAC_ALGO, secret).update(data).digest('hex');
    return `${data}.${sig}`;
}

/**
 * Verify a signed session cookie and return the parsed payload, or null if
 * the signature is invalid, the cookie is malformed, or the payload cannot be
 * parsed.
 *
 * The HMAC comparison uses `timingSafeEqual` to prevent timing attacks.
 *
 * @throws if SESSION_SECRET is not set or too short.
 */
export function parseSessionCookie<T>(value: string): T | null {
    if (!value) return null;

    const dotIdx = value.lastIndexOf('.');
    if (dotIdx < 1) return null;

    const data = value.slice(0, dotIdx);
    const sig = value.slice(dotIdx + 1);

    // Reject immediately if the signature is not the expected hex length.
    // (Avoids variable-length timingSafeEqual which requires equal-length buffers.)
    if (sig.length !== DIGEST_HEX_LEN) return null;

    try {
        const secret = getSecret();
        const expected = createHmac(HMAC_ALGO, secret).update(data).digest('hex');

        const sigBuf = Buffer.from(sig, 'hex');
        const expBuf = Buffer.from(expected, 'hex');

        // Both are 32 bytes because DIGEST_HEX_LEN is 64 — safe to compare.
        if (!timingSafeEqual(sigBuf, expBuf)) return null;

        const json = Buffer.from(data, 'base64url').toString('utf-8');
        return JSON.parse(json) as T;
    } catch {
        return null;
    }
}
