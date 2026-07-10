# Haven Alpha Success Metrics (R0)

These metrics are the gate the alpha must pass before the public beta
opens. They are longitudinal; the alpha collects data without trusting
event counts alone. Qualitative interviews are part of the measurement.

## Activation and retention proxies

- **Active notes**: number of unique documents touched (read or written
  by a human) per day. Proxy for daily-active usage.
- **Searches**: total queries against `search_brain` (in-app and via MCP)
  per day.
- **Chats with at least one citation click**: count of chats where the user
  clicked a `source_*` link. This is the real grounding signal.

Targets:

- D7 retention proxy: ≥ 4 active days for ≥ 60% of alpha cohort.
- Median number of citation-clicked chats / total chats ≥ 0.20 within
  the first 30 days.

## Safety and trust

- **Safe-vault-open completion rate**: percentage of external testers who
  reach the "first note is readable" state without losing data on a real
  vault. Target: ≥ 80%.
- **Zero destructive or silent rewrites**: a hard invariant. Per
  `ADR-003`, every write goes through the OKF linter + dual-identity
  pipeline. Any silent rewrite is a bug.
- **Approved patch acceptance rate**: percentage of agent-via-MCP patches
  the human approves. Useful signal, not a target.

## Setup and onboarding

- **Model setup completion rate**: percentage of users who finish the
  first-run benchmark and pick a tier. Target: ≥ 70%.
- **Time-to-first-cited-answer** (founder workflow): median ≤ 10 minutes
  on a real founder vault.
- **Import completion rate**: percentage of imports that finish with a
  clean import dashboard and zero non-conformant writes from Haven.
  Target: ≥ 90%.

## Interop

- **MCP connect success rate**: percentage of attempted MCP client
  connections that succeed without manual troubleshooting. Target: ≥ 95%.
- **Read MCP usage**: at least one Cursor/Claude Code session per user
  per week that calls `search_brain` or `read_document`.

## Failure analysis

Counts are not enough: the alpha **interviews non-activated testers**.
The events tell us who churned; the interviews tell us why.

## Reporting

- Aggregate metrics published weekly in `docs/research/alpha-metrics.md`.
- Per-test-session interviews kept in private qualitative notes; the
  synthesis is published in `docs/research/dogfood-report.md`.
