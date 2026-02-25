// RepoRoller Webhook Receiver — Go Example
// ==========================================
// Demonstrates how to receive and verify RepoRoller outbound webhook notifications.
//
// Requirements:
//
//	Go 1.21+  (only stdlib — no external dependencies)
//
// Usage:
//
//	WEBHOOK_SECRET="your-shared-secret-value" go run receiver.go
//
// The server listens on port 8080 and accepts POST /webhook requests.
//
// See docs/notifications.md for full webhook documentation.
package main

import (
	"crypto/hmac"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"log/slog"
	"net/http"
	"os"
	"strconv"
	"strings"
)

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

var webhookSecret []byte

func init() {
	secret := os.Getenv("WEBHOOK_SECRET")
	if secret == "" {
		slog.Error("WEBHOOK_SECRET environment variable is not set")
		os.Exit(1)
	}
	webhookSecret = []byte(secret)
}

func port() string {
	p := os.Getenv("PORT")
	if p == "" {
		return "8080"
	}
	if _, err := strconv.Atoi(p); err != nil {
		slog.Error("PORT must be a number", "value", p)
		os.Exit(1)
	}
	return p
}

// ---------------------------------------------------------------------------
// Event payload types
// ---------------------------------------------------------------------------

// RepositoryCreatedPayload contains all fields sent in a repository.created event.
// Optional fields use pointer types so absent fields deserialise as nil.
type RepositoryCreatedPayload struct {
	EventType        string            `json:"event_type"`
	EventID          string            `json:"event_id"`
	Timestamp        string            `json:"timestamp"`
	Organization     string            `json:"organization"`
	RepositoryName   string            `json:"repository_name"`
	RepositoryURL    string            `json:"repository_url"`
	RepositoryID     string            `json:"repository_id"`
	CreatedBy        string            `json:"created_by"`
	RepositoryType   *string           `json:"repository_type,omitempty"`
	TemplateName     *string           `json:"template_name,omitempty"`
	ContentStrategy  string            `json:"content_strategy"`
	Visibility       string            `json:"visibility"`
	Team             *string           `json:"team,omitempty"`
	Description      *string           `json:"description,omitempty"`
	CustomProperties map[string]string `json:"custom_properties,omitempty"`
}

// genericPayload is used only to read the event_type field before full deserialisation.
type genericPayload struct {
	EventType string `json:"event_type"`
}

// ---------------------------------------------------------------------------
// Signature verification
// ---------------------------------------------------------------------------

// verifySignature returns true when signatureHeader matches the HMAC-SHA256 of
// rawBody using the shared secret.
//
// Uses hmac.Equal (constant-time) to prevent timing attacks.
func verifySignature(rawBody []byte, signatureHeader string) bool {
	const prefix = "sha256="
	if !strings.HasPrefix(signatureHeader, prefix) {
		return false
	}

	receivedHex := signatureHeader[len(prefix):]
	receivedBytes, err := hex.DecodeString(receivedHex)
	if err != nil {
		return false
	}

	mac := hmac.New(sha256.New, webhookSecret)
	mac.Write(rawBody)
	computed := mac.Sum(nil)

	// Constant-time comparison — never use bytes.Equal here.
	return hmac.Equal(computed, receivedBytes)
}

// ---------------------------------------------------------------------------
// Event handlers
// ---------------------------------------------------------------------------

func handleRepositoryCreated(payload RepositoryCreatedPayload) {
	templateName := "(none)"
	if payload.TemplateName != nil {
		templateName = *payload.TemplateName
	}
	team := "(none)"
	if payload.Team != nil {
		team = *payload.Team
	}

	slog.Info("Repository created",
		"event_id", payload.EventID,
		"org", payload.Organization,
		"name", payload.RepositoryName,
		"url", payload.RepositoryURL,
		"created_by", payload.CreatedBy,
		"visibility", payload.Visibility,
		"template", templateName,
		"team", team,
		"strategy", payload.ContentStrategy,
	)

	// Add your integration logic here:
	//   - Post a Slack / Teams notification
	//   - Register the repo in a service catalog
	//   - Trigger a CI provisioning pipeline
	//   - Update an asset inventory database
}

// ---------------------------------------------------------------------------
// HTTP handler
// ---------------------------------------------------------------------------

func webhookHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method Not Allowed", http.StatusMethodNotAllowed)
		return
	}

	// Read the full body BEFORE parsing — signature covers the raw bytes.
	rawBody, err := io.ReadAll(io.LimitReader(r.Body, 1<<20)) // 1 MiB limit
	if err != nil {
		slog.Error("Failed to read request body", "error", err)
		http.Error(w, "Internal Server Error", http.StatusInternalServerError)
		return
	}
	defer r.Body.Close()

	// 1. Verify signature.
	sigHeader := r.Header.Get("X-RepoRoller-Signature-256")
	if !verifySignature(rawBody, sigHeader) {
		slog.Warn("Rejected request with invalid signature", "remote_addr", r.RemoteAddr)
		http.Error(w, "Unauthorized", http.StatusUnauthorized)
		return
	}

	// 2. Determine event type.
	var env genericPayload
	if err := json.Unmarshal(rawBody, &env); err != nil {
		slog.Error("Failed to parse JSON body", "error", err)
		http.Error(w, "Bad Request", http.StatusBadRequest)
		return
	}

	// 3. Dispatch on event type.
	switch env.EventType {
	case "repository.created":
		var payload RepositoryCreatedPayload
		if err := json.Unmarshal(rawBody, &payload); err != nil {
			slog.Error("Failed to deserialise repository.created payload", "error", err)
			http.Error(w, "Bad Request", http.StatusBadRequest)
			return
		}
		handleRepositoryCreated(payload)

	default:
		slog.Info("Ignoring unknown event type", "event_type", env.EventType)
	}

	// Always acknowledge promptly — processing is fire-and-forget from sender's perspective.
	w.WriteHeader(http.StatusNoContent)
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

func main() {
	addr := fmt.Sprintf("0.0.0.0:%s", port())

	mux := http.NewServeMux()
	mux.HandleFunc("/webhook", webhookHandler)

	slog.Info("RepoRoller webhook receiver listening", "addr", addr)
	// In production, terminate TLS at a reverse proxy / load balancer.
	// Your notifications.toml endpoint URL must use https://.
	if err := http.ListenAndServe(addr, mux); err != nil {
		slog.Error("Server failed", "error", err)
		os.Exit(1)
	}
}
