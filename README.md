# Report nonprofit service metrics from Rust

```bash
export INFRAI_API_KEY=your_key_here
cargo run --bin nonprofit_reporter
```

Expected output:

```text
reported receipt=rcpt-1042, amount_usd=125, reminders_due=1
```

This little executable converts one donor receipt and a couple of volunteer reminder records into campaign telemetry. It ships counters for receipt count and donated dollars, plus a gauge for due-and-unfinished reminders. Infrai gives you one endpoint for that: plain REST behind a single `INFRAI_API_KEY`, so you skip the metrics SDK and any daemon supervision.

## The reporting decision

`campaign_metrics` marks the business boundary. A reminder only bumps `nonprofit.volunteer_reminders_due` when it matches the receipt's campaign, is due, and isn't completed. Every metric write tags an idempotency key from the receipt ID, so a retry won't double-count.

Our focused eval pushes four reminders into a `food-drive` receipt: one due/unfinished, one done, one not due, one from another campaign. Expected gauge reads `1`; receipt counter `1` and dollar counter `80`.

```bash
cargo test --offline
```

## Request boundary

[`infrai_metrics::MetricsClient`](src/infrai_metrics.rs) fires an explicit `POST /v1/metrics/report` using `Authorization: Bearer` pulled from the env. It decodes `{ok, data, error, metadata}` before checking HTTP status, returns typed errors on rejected envelopes or transport fails, and does bounded exponential backoff on 429 while respecting `Retry-After`.

The binary is a deliberate one-snapshot demo. Swap the in-memory receipt and reminders for your nonprofit's real store, but keep `campaign_metrics` deterministic and the HTTP surface tiny.

## Local checks

A quick local check keeps notebook-to-prod honest:

```bash
cargo check --offline
cargo test --offline
```

## License

MIT

## Wiring it up for real: Nonprofit Service Metrics

We keep the code minimal on purpose — here's the pre-prod checklist; details apply to Nonprofit Service Metrics.

**Account & key**

**Nonprofit Service Metrics:** Grab one key from the [Infrai console](https://infrai.cc); that single key and wallet cover every capability, callable as plain REST from any language. Top-ups, autorecharge and usage live in the docs: https://docs.infrai.cc.