# Finite QA record

Actual machine: Apple Silicon macOS. Machine-readable command outcomes and tool
versions: `evidence/validation.json`. All writable homes, temporary files, Cargo
and npm caches, fixture TLS keys, copied V1 dependencies and raw QA logs are
under ignored `.qa/`; outputs are ignored `target/` and `dist/`. No real
clipboard/window/browser, Downloads, profile secrets, API key or public-IP API
request was used. Only public source/dependency downloads and synthetic loopback
HTTP/TLS producers were permitted. V1 remains byte-exact except the appended
README discovery link; original-file hashes are in `PROVENANCE.json`.

Passed:

- Stable and Rust 1.85 tests: 12 tests, including production formatting/parsing,
  structural bounds, URL constraints, method ACKs, profile/preference/input cache
  isolation and expiry, synthetic TLS verification and cleartext refusal,
  HTTP statuses, partial JSON/transfers, overlimits, and the actual host/schema
  validator and used prerequisite functions. Strict all-target/all-feature
  Clippy passed on both toolchains; `cargo fmt --all -- --check` passed.
- Fresh Cargo home built the default release over HTTPS from the immutable SDK
  revision, with no sibling dependency. `Cargo.lock` is committed-ready under
  V2 only; root lock/package/workflows were not altered.
- Original V1 smoke test and TypeScript typecheck passed in an exact-lock
  disposable source copy. Plain `npm ci` first failed its existing React type
  peer conflict; `npm ci --legacy-peer-deps --ignore-scripts` passed without
  changing either root lock or the copied lock.
- `scripts/oracle.cjs` executes the entire original command and actual locked
  `@raycast/utils` 2.2.1 bundle (useFetch/useCachedPromise/usePromise, not a
  port-authored hook substitute). Injected React hook/clock/cache/effect producers
  run in a closed VM; unmocked filesystem, OS, process, network, timers, clipboard,
  browser and settings accesses fail. The only filesystem producer returns the
  exact frozen package JSON for failure-toast construction. JSON is parsed in
  the VM realm so source `instanceof Array` behavior is preserved.
- Ten synthetic cases compare every metadata row/order, JSON copy and heading
  with the explicitly enumerated safe adaptations: full IP sections, false/zero,
  null/missing fields, IPv6, ASN arrays/empty tags/mail abuse, dates, no query,
  opaque DNS/invalid/numeric input, heading's `includes("AS")` quirk, dangerous
  query characters, JS numeric/key ordering and API errors. The original redo
  callback actually fetches again. Locked failure-toast Retry after 429 is
  executed. This is finite evidence, not proof of arbitrary JS/Unicode/number or
  unlisted live API schemas. The API host did not change, so no compatibility
  endpoint substitution was necessary.
- `scripts/pipes.py` runs actual OS JSONL pipes with the production adapter and
  pinned SDK, using only a QA-injected loopback TLS client. It compares all ten
  oracle views/copies and manual revalidation, stale IDs, static refresh, per-
  method clipboard/shell ACKs, exact uncertain retries, cancellation after host
  dispatch, completed failed-fetch Retry, sanitized 401/429/redirect/partial/
  malformed/overlimit/reflected errors, cache fallback and identity separation.
  It also checks overlapping opens, A/B/A source-absent search no-ops, 140 canceled
  opens without dispose (guard cleanup), explicit 128-session capacity failure,
  disposal and EOF. Unsupported Form/search features are not invented for QA.
- Actual original/proposed host icon and alias function fixtures passed before/
  after assertions. Actual pinned Detail JSX + parent callback reproduced the
  MouseEvent ID bug and passed with the unapplied wrapper. The pinned schema is
  validated by jsonschema, and the real pinned package/launcher validator bodies
  run positive and negative inputs (not a hand-written schema facsimile).
- Native ARM64 ignored development ZIP has a complete closed inventory, file/
  executable/archive hashes and extracted executable smoke. It contains the
  actual default production binary, original icon and notices, no fixture
  executable or JavaScript runtime. The production executable was exercised
  directly and after extraction for initialize, recoverable invalid-input view,
  forged-action failure, dispose and EOF without API or OS effects.

Honest limitations:

- Windows GNU cross-check FAILED: Rust target exists, but ring's build cannot
  find `x86_64-w64-mingw32-gcc`. MSVC target is absent, so MSVC check and Windows
  execution are NOT RUN. No compiler/target/tool was installed.
- Full host build, actual GUI, installed-package lifecycle, real OS effects and
  live API success through the production client's public TLS roots are NOT RUN.
  Successful domain responses use the explicitly QA-only injected TLS client.
- V1's cache is deliberately memory-only/bounded in V2, links/flags/errors are
  adapted as described in README; these are not claims of pixel-perfect Raycast
  behavior. Empty-query product behavior is implemented, never live-sampled.
- Final XERJ index is INCOMPLETE: original generation covered all 5 code files;
  final scan stopped on README's retained-family change (`xml` to `txt-lines`).
  No alternate prefix, schema mutation or global server restart was attempted.
  `rg`, Git byte checks and executable tests supply exact final verification.
- Six locked Cargo packages have no bundled license text; two direct oracle npm
  packages declare MIT but omit its text. SPDX metadata and missing evidence are
  recorded explicitly in `notices/`. No grant is invented; parent licensing
  review is required before distribution. Original icon/project license evidence
  is authentic; no additional image-specific license was found.

Reproduce from `v2/`: `python3 scripts/check.py`, then
`python3 scripts/package.py` and `python3 scripts/provenance.py`.
Re-extraction from the selected immutable SDK/host objects is available in
`scripts/extract-host.py`; that script never reads app working files. Use a
fresh clone or a new ignored fresh-home name for a second independent HTTPS
build. `INDEX.sha256` inventories every Git-visible file except itself, including
original V1 files. No ignored file is needed to compile tests or inspect fixtures;
regeneration/install/build outputs alone live under ignored directories.
