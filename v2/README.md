# ipapi.is — standalone Rust V2

Development/review implementation on `feat/native-v2-rust`. Not installed,
published, released, or catalog-pinned. The source manifest deliberately has
`artifacts: {}`. Keep V1 as the default until the parent completes review,
prerequisites, platform acceptance and a separately authorized publication.

The entire declared `lookup-ip-or-asn` command is implemented as a standalone
schema-3 / protocol-1 Rust process. It takes the original optional `ip` launch
argument and automatically requests `https://api.ipapi.is?q=<encoded argument>`.
Empty input intentionally means the service's current-IP lookup in the product;
QA substitutes synthetic responses and never calls that endpoint. There is no
source API key preference, local DNS/IP parser, Form, List search, local network
scan, reverse DNS, pagination, offline dataset, history or telemetry.

The source treats input as opaque: it does not trim it or require valid IP/ASN
syntax. Its heading test is literally `ip.toUpperCase().includes("AS")`, including
false positives; numeric ASNs, IPv6, DNS-shaped and invalid input still go to the
same API. API response shape (`ip` property present vs absent), rather than that
heading test, selects IP/ASN metadata. Dates remain service strings; zero-valued
latitude, longitude, UNIX time and VPN last-seen display `N/A`, exactly as V1.
Optional missing labels remain missing. `false`, empty arrays and zero ASN are
not replaced with invented data. Metadata sections and the original Copy JSON /
Revalidate action order are preserved. Copy uses JS-compatible number formatting
and property order; finite oracle coverage is documented in [QA.md](QA.md).

The exact original Raycast PNG is reused for the extension and command, with no
conversion. Pins and file hashes are in [PROVENANCE.json](PROVENANCE.json).

Adaptations required by the selected host/security contract:

- Query values are encoded as one `q` value. V1 raw interpolation could turn `&`
  or `#` into URL syntax. Markdown headings encode special input characters;
  API labels are structured plain text.
- Metadata links become plain labels plus explicit, revision-bound Open actions
  after the original two actions. Only validated HTTP(S) destinations are
  offered; email/phone stay readable/copyable in JSON. No renderer `href` can
  bypass adapter validation. Credential-bearing URLs and reflected auth query
  parameters/fragments are rejected. Company domain display keeps V1's peculiar
  case-sensitive `includes("http")` rule.
- The pinned metadata icon renderer cannot display remote flags or distinguish
  Check/Xmark reliably. Country uses the original Map fallback, without a flag
  request; boolean labels also show Yes/No.
- Service errors are displayed automatically as a generic error in the Detail,
  including sanitized JSON copy; raw API/transport/host error strings never
  become Markdown, logs, views or copied error text. Failed fetches expose Retry.
  Raycast's automatic failure-toast report/copy-logs UI is adapted to this error
  view; no Raycast issue-report link or raw log copy is added.
- V1's unexpired persistent Raycast cache becomes a bounded, private in-memory
  cache (32 responses, 60 seconds). Opening always revalidates. A recent exact
  profile/preference/input match is shown with an explicit stale notice only if
  the new fetch fails. Manual Revalidate always fetches. Process exit clears it;
  no private storage or V1 migration is necessary. No auth preference exists.
- Views use `refreshMs: null`. SDK transactional state is supplemented with
  authoritative, consumed-before-await action IDs and a session-owned Drop
  guard. Unknown/stale actions cannot cause effects. Unconfirmed clipboard/open
  effects expose only explicit exact-byte Retry; refresh does not replay or
  refetch. Cancellation cannot undo a request or an already delivered OS effect.

TLS verification stays enabled. Product origins are fixed HTTPS; proxies and
redirects are disabled. Connect/request/total HTTP deadlines are 5/15/16 seconds;
host effects are bounded to 8 seconds. Limits: 1 MiB body, 24 nested levels,
16,384 JSON values, 1,024 entries/array, 16 KiB/string and 1,024-byte input.
Overlimits fail visibly, never as silently truncated success. The SDK bounds
sessions, handler/output queues and reverse RPC waiters. The only pending
session task is the SDK-owned lifetime guard, with no polling or detached work.

Build from `v2/` with `cargo build --locked --release` (Rust 1.85 minimum).
The SDK is an HTTPS Git dependency pinned to
`e955911f0bf5b558f9d3eb5183df0adc95c22af2`, never a local path, app or Tauri.
`python3 scripts/check.py` reproduces checks under ignored `.qa/`; it requires
already available stable/1.85 toolchains, Node/npm, Python and OpenSSL, and does
not install toolchains. The original V1 lock needs npm's `--legacy-peer-deps`
for its existing React type peer conflict; the disposable copy alone is used.
`python3 scripts/package.py` builds an ignored development ZIP from the fresh
release build and verifies its extracted executable. CI example is inactive in
`ci/check.yml.example` and publishes nothing.

See [INTEGRATION.md](INTEGRATION.md) for UNAPPLIED host prerequisites and
[QA.md](QA.md) for actual results and unrun acceptance. License/notice evidence,
including missing package text evidence, is explicit in [NOTICE](NOTICE).
