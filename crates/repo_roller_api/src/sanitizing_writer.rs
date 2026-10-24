//! Shared `std::io::Write` / `tracing_subscriber::fmt::MakeWriter` wrappers
//! that redact sensitive field values (via
//! [`crate::logging::redact_sensitive_fields`]) before any formatted log
//! line reaches the underlying writer.
//!
//! Extracted from `logging.rs` (Task #1) during Task #2's REFACTOR pass:
//! `logging.rs`'s `build_sanitizing_json_subscriber` (a full `fmt::Subscriber`)
//! and `tracing_otel.rs`'s `build_composed_subscriber_for_test`/
//! `init_tracing_with_otel` (a `Registry`-based `Layer` composition, needed
//! so an OTel export layer can be attached via `.with()`) build genuinely
//! different `Subscriber` shapes -- see `tracing_otel.rs`'s module doc
//! comment for why a `fmt::Subscriber` cannot itself be layered under
//! `Registry::with()`. But the writer-wrapping *types* themselves have no
//! dependency on which subscriber shape they end up in: they are pure
//! generic decorators over any `W`. Keeping two byte-for-byte-identical
//! copies of these types (one per module) would have been undetected drift
//! waiting to happen, so both modules import from here instead.
//!
//! Fields are `pub(crate)` (not private) specifically so `logging_tests.rs`
//! (a descendant module of `logging`, not of this module) can keep
//! constructing `SanitizingWriter { inner }` by struct literal directly, as
//! it did before this extraction -- this module intentionally exposes no
//! *public* (crate-external) API surface, only crate-internal reuse.

/// A `std::io::Write` wrapper that redacts sensitive field values (via
/// [`crate::logging::redact_sensitive_fields`]) before forwarding bytes to
/// the wrapped writer.
///
/// Assumes (like the redaction function it delegates to) that each `write()`
/// call receives one complete, newline-terminated formatted log line -- see
/// `logging.rs`'s module docs for the known gap if that assumption is ever
/// violated (e.g. by a batching/splitting `MakeWriter`).
pub(crate) struct SanitizingWriter<W> {
    pub(crate) inner: W,
}

impl<W: std::io::Write> std::io::Write for SanitizingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let text = String::from_utf8_lossy(buf);
        let (body, had_trailing_newline) = match text.strip_suffix('\n') {
            Some(stripped) => (stripped, true),
            None => (text.as_ref(), false),
        };

        let mut sanitized = crate::logging::redact_sensitive_fields(body).into_bytes();
        if had_trailing_newline {
            sanitized.push(b'\n');
        }

        self.inner.write_all(&sanitized)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

/// A `tracing_subscriber::fmt::MakeWriter` that wraps every writer it
/// produces in a [`SanitizingWriter`].
#[derive(Clone)]
pub(crate) struct SanitizingMakeWriter<W> {
    pub(crate) inner: W,
}

impl<'a, W> tracing_subscriber::fmt::MakeWriter<'a> for SanitizingMakeWriter<W>
where
    W: tracing_subscriber::fmt::MakeWriter<'a>,
{
    type Writer = SanitizingWriter<W::Writer>;

    fn make_writer(&'a self) -> Self::Writer {
        SanitizingWriter {
            inner: self.inner.make_writer(),
        }
    }
}
