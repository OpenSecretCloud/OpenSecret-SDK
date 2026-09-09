# OpenSecret SDKs

> **Development moved to [MaplePrivacyLabs/Maple](https://github.com/MaplePrivacyLabs/Maple/tree/master/sdk).**
> New SDK work, issues, and pull requests belong in the monorepo's `sdk/`
> (TypeScript/React) and `sdk/rust/` (Rust) directories.

The current packages are [`@mapleai/sdk`](https://www.npmjs.com/package/@mapleai/sdk)
and [`maple-sdk`](https://crates.io/crates/maple-sdk), first published as
3.5.2 and 3.6.2 respectively. See the
[current SDK documentation](https://github.com/MaplePrivacyLabs/Maple/blob/master/sdk/README.md)
for TypeScript installation and migration, and the
[current Rust SDK guide](https://github.com/MaplePrivacyLabs/Maple/blob/master/sdk/rust/README.md)
for Rust. Exported names such as `OpenSecretProvider` and `OpenSecretClient`
remain unchanged; Rust imports use `maple_sdk`.

Existing `@opensecret/react` and `opensecret` package versions remain available
for older consumers. Updating source repositories does not update those
consumers automatically. New package releases use the monorepo's independent,
protected [SDK publishing workflows](https://github.com/MaplePrivacyLabs/Maple/blob/master/docs/sdk-publishing.md).
Do not publish new SDK versions from this checkout.

The source and all retained component documentation, including `rust/README.md`,
describe the legacy packages and remain as historical reference. Existing
issues and draft pull requests remain here
until their disposition is recorded; this notice does not close or port them.

## Legacy SDK reference

This repository contains the TypeScript/React and Rust clients used by Maple
and OpenSecret's internal applications. Both clients establish attested,
end-to-end encrypted sessions with an OpenSecret backend and expose the API
surface needed by those applications.

The developer/platform API remains part of the TypeScript SDK for internal
OpenSecret workflows. This repository does not maintain or deploy a separate
documentation website; keep behavior documentation close to the exported code
and tests.

## Repository layout

- `src/` — `@opensecret/react`, including the React providers, encrypted API
  client, attestation policy, model/conversation APIs, and internal developer
  platform client.
- `rust/` — the `opensecret` crate used by native clients.
- `docs/PLATFORM.md` — internal developer/platform API notes.
- `.github/workflows/` — TypeScript and Rust validation.

## Security model

For non-local endpoints, both SDKs require HTTPS, verify AWS Nitro attestation,
and enforce an environment-scoped PCR0 trust policy before completing key
exchange. Official PCR0 histories are signed and bundled with the SDKs.

Mock attestation is limited to exact loopback development endpoints (plus the
documented Android emulator alias in the Rust SDK). Do not weaken attestation,
PCR0 validation, or encrypted transport to accommodate a caller.

The SDKs use operating-system or Web Crypto randomness for keys, nonces, and
session material. Never substitute deterministic or convenience randomness in
production paths.

## TypeScript/React SDK

Install the package:

```sh
bun add @opensecret/react
```

Wrap the application with `OpenSecretProvider` and supply the backend URL and
client ID:

```tsx
import { OpenSecretProvider } from "@opensecret/react";
import type { ReactNode } from "react";

export function AppProviders({ children }: { children: ReactNode }) {
  return (
    <OpenSecretProvider
      apiUrl="https://api.example.com"
      clientId="00000000-0000-0000-0000-000000000000"
      pcrConfig={{ environment: "production" }}
    >
      {children}
    </OpenSecretProvider>
  );
}
```

Use `useOpenSecret` for authentication, encrypted application APIs,
conversations, inference, and account operations. Internal developer tooling
uses `OpenSecretDeveloper` and `useOpenSecretDeveloper`; preserve that surface
when changing the public exports.

### Development

Use the pinned Nix shell and Bun version:

```sh
nix develop --no-update-lock-file
bun install --frozen-lockfile --ignore-scripts
bun run format:check
bun run build
bun test --timeout 30000
```

Live integration tests read the variables documented in `.env.example`. Use
disposable test accounts and never commit credentials.

Inspect the publishable npm artifact with:

```sh
bun run pack
```

Only `dist/` is included in the package.

The legacy `just publish-npm` recipe is retained in source for history.
Use the [monorepo publishing guide](https://github.com/MaplePrivacyLabs/Maple/blob/master/docs/sdk-publishing.md)
for new versions.

## Rust SDK

Add the crate to a Rust application:

```toml
[dependencies]
opensecret = "3"
```

The primary entry point is `OpenSecretClient`. See `rust/README.md` for native
client examples and transport details.

Run the Rust validation from the repository root:

```sh
nix develop --no-update-lock-file -c bash -lc '
  cd rust
  cargo fmt --all -- --check
  cargo clippy --locked --all-targets --all-features -- -D warnings
  cargo test --locked --all-features
  cargo doc --locked --no-deps --all-features
'
```

Integration tests use the variables documented in `rust/.env.example` and are
separate from the default local validation path.

The legacy `just publish-cargo` recipe is retained in source for history.
Use the [monorepo publishing guide](https://github.com/MaplePrivacyLabs/Maple/blob/master/docs/sdk-publishing.md)
for new versions.

## Change discipline

- Keep the TypeScript and Rust attestation policies aligned intentionally;
  neither SDK's passing tests prove parity with the other.
- Treat API compatibility, authentication state, encrypted retry behavior, and
  PCR policy changes as security-sensitive.
- Update source comments and focused tests with behavior changes instead of
  regenerating a standalone documentation site.
- Validate the built npm package and Rust crate boundary before publishing a
  release.

## License

MIT
