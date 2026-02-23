"""
RepoRoller Webhook Receiver — Python Example
============================================
Demonstrates how to receive and verify RepoRoller outbound webhook notifications.

Requirements:
    pip install flask

Usage:
    export WEBHOOK_SECRET="your-shared-secret-value"
    python receiver.py

The server listens on port 8080 and accepts POST /webhook requests.

See docs/notifications.md for full webhook documentation.
"""

import hashlib
import hmac
import json
import logging
import os
import sys

from flask import Flask, abort, request

app = Flask(__name__)
logging.basicConfig(level=logging.INFO, format="%(levelname)s  %(message)s")
logger = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

WEBHOOK_SECRET = os.environ.get("WEBHOOK_SECRET")
if not WEBHOOK_SECRET:
    logger.error("WEBHOOK_SECRET environment variable is not set")
    sys.exit(1)


# ---------------------------------------------------------------------------
# Signature verification
# ---------------------------------------------------------------------------


def verify_signature(raw_body: bytes, signature_header: str) -> bool:
    """Return True if *signature_header* matches the HMAC-SHA256 of *raw_body*.

    Uses constant-time comparison (``hmac.compare_digest``) to prevent timing
    attacks.

    Args:
        raw_body: The raw, unparsed request body bytes.
        signature_header: Value of the ``X-RepoRoller-Signature-256`` header,
            expected format ``sha256=<hex>``.

    Returns:
        ``True`` if the signature is valid, ``False`` otherwise.
    """
    if not signature_header or not signature_header.startswith("sha256="):
        return False

    expected_hex = signature_header[len("sha256=") :]

    mac = hmac.new(
        WEBHOOK_SECRET.encode("utf-8"),
        msg=raw_body,
        digestmod=hashlib.sha256,
    )
    computed_hex = mac.hexdigest()

    # Constant-time comparison — never use == here.
    return hmac.compare_digest(computed_hex, expected_hex)


# ---------------------------------------------------------------------------
# Webhook endpoint
# ---------------------------------------------------------------------------


@app.route("/webhook", methods=["POST"])
def webhook():
    """Handle incoming RepoRoller webhook notifications."""
    # Read raw bytes BEFORE parsing — signature covers the raw body.
    raw_body = request.get_data()
    signature = request.headers.get("X-RepoRoller-Signature-256", "")

    # 1. Verify signature.
    if not verify_signature(raw_body, signature):
        logger.warning(
            "Rejected request with invalid signature from %s",
            request.remote_addr,
        )
        abort(401)

    # 2. Parse payload.
    try:
        payload = json.loads(raw_body)
    except json.JSONDecodeError as exc:
        logger.error("Failed to parse JSON body: %s", exc)
        abort(400)

    event_type = payload.get("event_type")

    # 3. Dispatch on event type.
    if event_type == "repository.created":
        handle_repository_created(payload)
    else:
        logger.info("Ignoring unknown event type: %s", event_type)

    # Always acknowledge promptly — processing is fire-and-forget from sender's perspective.
    return "", 204


# ---------------------------------------------------------------------------
# Event handlers
# ---------------------------------------------------------------------------


def handle_repository_created(payload: dict) -> None:
    """Process a ``repository.created`` event.

    Args:
        payload: Parsed event payload dict.
    """
    event_id = payload.get("event_id", "unknown")
    org = payload.get("organization", "")
    repo_name = payload.get("repository_name", "")
    repo_url = payload.get("repository_url", "")
    created_by = payload.get("created_by", "")
    visibility = payload.get("visibility", "")
    template = payload.get("template_name")
    team = payload.get("team")

    logger.info(
        "Repository created — event_id=%s org=%s name=%s url=%s "
        "created_by=%s visibility=%s template=%s team=%s",
        event_id,
        org,
        repo_name,
        repo_url,
        created_by,
        visibility,
        template or "(none)",
        team or "(none)",
    )

    # Add your integration logic here:
    #   - Post a Slack / Teams notification
    #   - Register the repo in a service catalog
    #   - Trigger a CI provisioning pipeline
    #   - Update an asset inventory database


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    port = int(os.environ.get("PORT", "8080"))
    logger.info("Starting RepoRoller webhook receiver on port %d", port)
    # In production use a proper WSGI server (gunicorn, uvicorn, etc.)
    # and terminate TLS at a reverse proxy / load balancer.
    app.run(host="0.0.0.0", port=port)
