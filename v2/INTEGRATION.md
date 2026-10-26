# Parent integration review — UNAPPLIED

Canonical ID: `com.tezbar.native.ipapi-is`. Exact replacements:
`com.tezbar.ipapi-is` (own pinned manifest) and `raycast.ipapi-is` (confirmed host
selection). Only `lookup-ip-or-asn` is covered. No app, profile, SDK, catalog,
ledger or sibling repository is modified. No install/release URL is invented.

Host selection is frozen at `ea100cee3f69711de12485ba061950c5fc65258f`.
SDK selection is frozen at `e955911f0bf5b558f9d3eb5183df0adc95c22af2` (0.1.1).
The three exact SDK documents are under `evidence/sdk/`; the host source-object
hashes and exact function extracts are under `evidence/host/`. Extraction reads
immutable Git objects and writes only in this clone. There is no app/Tauri build
dependency and no JavaScript runtime in the product binary.

Only used generic prerequisites are proposed:

1. `host-prerequisites/registry.patch` separates the verified active package's
   resource directory (`Package.dir`) from canonical `Installed.path`. Extension
   and command icons must resolve from the active version while preferences and
   uninstall retain their canonical roots. It also allows an exact `replaces`
   match from a verified canonical manifest to select a covered, compatible Rust
   command. It does not broaden a prefix match or route uncovered commands.
2. `host-prerequisites/detail-primary.patch` wraps the Detail button callback as
   `() => onRunPrimaryAction()`. The pinned button passes its MouseEvent directly;
   the parent callback otherwise treats it as an action ID. The actual extracted
   JSX and parent callback reproduce this before and pass after the wrapper.

These patches have NOT been applied to the app. `tests/host.rs` compiles the
original and proposed narrow functions with explicit dependency fixtures.
`scripts/renderer.cjs` executes the actual before/after Detail callback. The
full pinned host package/launcher validator and JSON Schema are also exercised,
including negative cases. These narrow tests do not establish a full host build,
GUI behavior, preference retention or real install/uninstall acceptance.

The only used OS methods are:

| Method | Parameters | Exact selected result contract |
| --- | --- | --- |
| `host.clipboard.write` | `{text: string}` | closed `{ok:true,message?:string}` |
| `host.shell.open` | `{target: string}` | closed `{ok:true,message?:string}` |

`host-method-producers.txt` contains their actual pinned router/platform
extracts; `host-replies.json` is derived from the actual return literals. The
pipe producer loads that file. Each method is separately exercised, including
null/contradiction rejection and uncertain exact retries. Null, booleans, extra
properties, `{ok:false}`, or non-string messages cannot confirm delivery.
Host errors remain generic and never echo their raw text.

No clipboard read, pasteText, visibility, settings, Form, controlled List
selection/dropdown, or no-view action is used. Accordingly no patches or generic
fabricated ACK tests for those methods are included. In particular the pinned
visibility method's null result is NOT changed. The command always returns a
Detail; ordinary Back/session disposal remains the SDK/host lifecycle contract.
Preferences and launch arguments are immutable per SDK session; no settings
adaptation or fake key preference is offered.

The QA-only `pipe-fixture` example injects a TLS client trusting a throwaway
loopback producer. It runs the same adapter and SDK through OS pipes but is
excluded from the development package. The product entrypoint accepts no
producer URL, certificate, proxy, fixture path or alternate origin. It is also
smoked directly and after extraction with recoverable input-validation/failure
paths that cannot contact the public API. Real successful API acceptance through
the product's public TLS roots remains unrun.

Parent must review the complete source diff/commits, licensing gaps and host
prerequisites, then separately authorize commit/push and remote verification.
Keep this clone intact until that review. Native Windows/GUI/live-service
acceptance and trusted artifact/catalog publication are future separate steps.
