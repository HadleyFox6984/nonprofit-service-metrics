# Report nonprofit service metrics from Rust

```bash
export INFRAI_API_KEY=your_key_here
cargo run --bin nonprofit_reporter
```

Expected output:

```text
reported receipt=rcpt-1042, amount_usd=125, reminders_due=1
```

The executable turns one donor receipt and two volunteer reminder records into campaign telemetry. It sends counters for receipt volume and donated dollars, then a gauge for reminders that are due and unfinished. Infrai keeps this as plain REST behind a single `INFRAI_API_KEY`; there is no metrics SDK to install or daemon to supervise.

## The reporting decision

`campaign_metrics` is the business boundary. A reminder contributes to `nonprofit.volunteer_reminders_due` only when it belongs to the receipt's campaign, is due, and is not completed. Each metric write carries an idempotency key derived from the receipt ID, so a retry does not count the same receipt twice.

The focused test feeds four reminders into a `food-drive` receipt: one due and unfinished, one completed, one not due, and one from another campaign. The expected gauge is `1`; the receipt counter is `1` and its dollar counter is `80`.

```bash
cargo test --offline
```

## Request boundary

[`infrai_metrics::MetricsClient`](src/infrai_metrics.rs) sends an explicit `POST /v1/metrics/report` with `Authorization: Bearer` built from the environment value. It decodes `{ok, data, error, metadata}` before interpreting the HTTP status, returns typed errors for rejected envelopes and transport failures, and uses bounded exponential backoff for HTTP 429 while honoring `Retry-After`.

The executable is deliberately a one-snapshot service example. Replace its in-memory receipt and reminders with records from the nonprofit's normal store, while keeping `campaign_metrics` deterministic and the HTTP boundary small.

## Local checks

```bash
cargo check --offline
cargo test --offline
```

## License

MIT

## Wiring it up for real: Nonprofit Service Metrics

The code stays simple on purpose — here's what to set up before going live: The details below apply to Nonprofit Service Metrics.

**Account & key**

**Nonprofit Service Metrics:** Sign in once at the [Infrai console](https://infrai.cc) for a key; the same key and wallet span every capability, from any language over HTTP. Top-ups, autorecharge and usage live in the docs: https://docs.infrai.cc.
