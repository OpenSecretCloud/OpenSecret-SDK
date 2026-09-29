# OpenSecret SDKs — development moved

This repository is archived. SDK development, issues, pull requests, and
publishing now live in [MaplePrivacyLabs/Maple](https://github.com/MaplePrivacyLabs/Maple/tree/master/sdk).

- **TypeScript/React:** [`@mapleai/sdk`](https://www.npmjs.com/package/@mapleai/sdk) —
  [installation and migration guide](https://github.com/MaplePrivacyLabs/Maple/blob/master/sdk/README.md).
- **Rust:** [`maple-sdk`](https://crates.io/crates/maple-sdk) —
  [Rust SDK guide](https://github.com/MaplePrivacyLabs/Maple/blob/master/sdk/rust/README.md).
- **Publishing:** [protected SDK release workflows](https://github.com/MaplePrivacyLabs/Maple/blob/master/docs/sdk-publishing.md).
- **New work:** [Maple issues](https://github.com/MaplePrivacyLabs/Maple/issues).

The legacy source, Git history, tags, releases, and license remain here for
compatibility and reference. Existing `@opensecret/react` and `opensecret`
package versions remain available; archiving this repository does not upgrade
existing consumers. Do not publish new versions from this checkout. See the
[legacy README](https://github.com/OpenSecretCloud/OpenSecret-SDK/blob/a657a5cc11a881aba761071215ee7c488eb62c91/README.md)
for the old package documentation.

## Preserved work

These unfinished items remain as historical references. Archival does not
mean they were implemented or resolved. Any continuation belongs in Maple,
with a link back to the original item.

- [Draft PR #102: Maple device pairing client](https://github.com/OpenSecretCloud/OpenSecret-SDK/pull/102).
- [Draft PR #89: Sigstore snapshot verification](https://github.com/OpenSecretCloud/OpenSecret-SDK/pull/89).
- [Issue #92: session-loss recovery in createCustomFetch](https://github.com/OpenSecretCloud/OpenSecret-SDK/issues/92).

## License

[MIT](LICENSE.md); the Rust SDK also retains its [license](rust/LICENSE).
