// RepoRoller Webhook Receiver — C# Example
// ==========================================
// Demonstrates how to receive and verify RepoRoller outbound webhook notifications.
//
// Requirements:
//   .NET 8 SDK or later
//
// Usage:
//   set WEBHOOK_SECRET=your-shared-secret-value
//   dotnet run
//
// The server listens on port 8080 and accepts POST /webhook requests.
//
// See docs/notifications.md for full webhook documentation.

using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Http;
using Microsoft.Extensions.Logging;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

string? webhookSecretStr = Environment.GetEnvironmentVariable("WEBHOOK_SECRET");
if (string.IsNullOrEmpty(webhookSecretStr))
{
    Console.Error.WriteLine("ERROR: WEBHOOK_SECRET environment variable is not set");
    return 1;
}

byte[] webhookSecret = Encoding.UTF8.GetBytes(webhookSecretStr);

int port = 8080;
if (Environment.GetEnvironmentVariable("PORT") is { } portStr && int.TryParse(portStr, out int p))
    port = p;

// ---------------------------------------------------------------------------
// Application setup
// ---------------------------------------------------------------------------

WebApplicationBuilder builder = WebApplication.CreateBuilder(args);
builder.Logging.ClearProviders();
builder.Logging.AddConsole();
builder.WebHost.UseUrls($"http://0.0.0.0:{port}");

WebApplication app = builder.Build();
ILogger logger = app.Logger;

logger.LogInformation("RepoRoller webhook receiver listening on port {Port}", port);
// In production, terminate TLS at a reverse proxy / load balancer.
// Your notifications.toml endpoint URL must use https://.

// ---------------------------------------------------------------------------
// Webhook endpoint
// ---------------------------------------------------------------------------

app.MapPost("/webhook", async (HttpContext ctx) =>
{
    // Read the full body BEFORE parsing — signature covers the raw bytes.
    using MemoryStream ms = new();
    await ctx.Request.Body.CopyToAsync(ms);
    byte[] rawBody = ms.ToArray();

    // 1. Verify signature.
    string sigHeader = ctx.Request.Headers["X-RepoRoller-Signature-256"].ToString();
    if (!VerifySignature(rawBody, sigHeader, webhookSecret))
    {
        logger.LogWarning("Rejected request with invalid signature from {Remote}",
            ctx.Connection.RemoteIpAddress);
        return Results.Unauthorized();
    }

    // 2. Parse payload.
    JsonDocument doc;
    try
    {
        doc = JsonDocument.Parse(rawBody);
    }
    catch (JsonException ex)
    {
        logger.LogError(ex, "Failed to parse JSON body");
        return Results.BadRequest("Invalid JSON");
    }

    // 3. Dispatch on event type.
    string eventType = doc.RootElement.GetProperty("event_type").GetString() ?? string.Empty;
    switch (eventType)
    {
        case "repository.created":
            RepositoryCreatedPayload? payload = JsonSerializer.Deserialize(
                rawBody, RepoRollerJsonContext.Default.RepositoryCreatedPayload);
            if (payload is null)
            {
                logger.LogError("Failed to deserialise repository.created payload");
                return Results.BadRequest("Invalid payload");
            }
            HandleRepositoryCreated(payload, logger);
            break;

        default:
            logger.LogInformation("Ignoring unknown event type: {EventType}", eventType);
            break;
    }

    // Always acknowledge promptly — processing is fire-and-forget from sender's perspective.
    return Results.NoContent();
});

await app.RunAsync();
return 0;

// ---------------------------------------------------------------------------
// Signature verification
// ---------------------------------------------------------------------------

/// <summary>
/// Returns true when <paramref name="signatureHeader"/> matches the HMAC-SHA256
/// of <paramref name="body"/> using <paramref name="secret"/>.
/// Uses <see cref="CryptographicOperations.FixedTimeEquals"/> to prevent timing attacks.
/// </summary>
static bool VerifySignature(byte[] body, string signatureHeader, byte[] secret)
{
    const string prefix = "sha256=";
    if (!signatureHeader.StartsWith(prefix, StringComparison.Ordinal))
        return false;

    string hexPart = signatureHeader[prefix.Length..];

    byte[] received;
    try   { received = Convert.FromHexString(hexPart); }
    catch { return false; } // malformed hex

    using HMACSHA256 hmac = new(secret);
    Span<byte> computed = stackalloc byte[32]; // SHA-256 is always 32 bytes
    hmac.TryComputeHash(body, computed, out _);

    // Constant-time comparison — never use SequenceEqual here.
    return CryptographicOperations.FixedTimeEquals(computed, received);
}

// ---------------------------------------------------------------------------
// Event handlers
// ---------------------------------------------------------------------------

static void HandleRepositoryCreated(RepositoryCreatedPayload payload, ILogger logger)
{
    logger.LogInformation(
        "Repository created: event_id={EventId} org={Org} name={Name} url={Url} " +
        "created_by={CreatedBy} visibility={Visibility} template={Template} " +
        "team={Team} strategy={Strategy}",
        payload.EventId,
        payload.Organization,
        payload.RepositoryName,
        payload.RepositoryUrl,
        payload.CreatedBy,
        payload.Visibility,
        payload.TemplateName ?? "(none)",
        payload.Team ?? "(none)",
        payload.ContentStrategy);

    // Add your integration logic here:
    //   - Post a Slack / Teams notification
    //   - Register the repo in a service catalog
    //   - Trigger a CI provisioning pipeline
    //   - Update an asset inventory database
}

// ---------------------------------------------------------------------------
// Payload types
// ---------------------------------------------------------------------------

/// <summary>All fields sent in a <c>repository.created</c> event.</summary>
public sealed record RepositoryCreatedPayload(
    [property: JsonPropertyName("event_type")]    string EventType,
    [property: JsonPropertyName("event_id")]      string EventId,
    [property: JsonPropertyName("timestamp")]     string Timestamp,
    [property: JsonPropertyName("organization")]  string Organization,
    [property: JsonPropertyName("repository_name")] string RepositoryName,
    [property: JsonPropertyName("repository_url")]  string RepositoryUrl,
    [property: JsonPropertyName("repository_id")]   string RepositoryId,
    [property: JsonPropertyName("created_by")]    string CreatedBy,
    [property: JsonPropertyName("content_strategy")] string ContentStrategy,
    [property: JsonPropertyName("visibility")]    string Visibility,
    [property: JsonPropertyName("repository_type")] string? RepositoryType = null,
    [property: JsonPropertyName("template_name")]   string? TemplateName = null,
    [property: JsonPropertyName("team")]            string? Team = null,
    [property: JsonPropertyName("description")]     string? Description = null,
    [property: JsonPropertyName("custom_properties")] Dictionary<string, string>? CustomProperties = null
);

// Source-generated JSON serialiser context — avoids reflection, compatible with Native AOT.
[JsonSerializable(typeof(RepositoryCreatedPayload))]
internal partial class RepoRollerJsonContext : JsonSerializerContext { }
