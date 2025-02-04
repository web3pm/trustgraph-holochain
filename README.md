<h1 align="center">
  <div>👋 Welcome to the</div>
  <img src="./doc/img/logo.png" alt="Logo" height="125">
  <div>Holochain library</div>
</h1>

<div align="center">

[![license](https://img.shields.io/github/license/trustgraph/trustgraph-holochain.svg?style=flat-square)](LICENSE.md)
[![PRs welcome](https://img.shields.io/badge/PRs-welcome-ff69b4.svg?style=flat-square)](https://github.com/trustgraph/trustgraph-holochain/issues?q=is%3Aissue+is%3Aopen+label%3A%22help+wanted%22)
[![made with love](https://img.shields.io/badge/made%20with%20%E2%99%A5%20-cc14cc.svg?style=flat-square)](https://github.com/trustgraph)

</div>

TrustGraph::Holochain is a Rust library, intended to allow for [Hololchain](https://www.holochain.org) developers to easily use the [Trust Graph](https://trustgraph.net/) protocol in their Happs.

_TrustGraph::Holochain is a very young codebase; **expect limited functionality**, and don’t use it in production just yet -- but do come collaborate and play as we develop it!_

## Prerequisites

- rust >= 1.56

## Install

In your `Cargo.toml`:

```rs
trust_atom = {git = "https://github.com/trustgraph/trustgraph-holochain.git", rev="v1.2.3", package = "trust_atom"}
```

Replace `v1.2.3` with the tag corresponding to the version you want. See the list of [available tags](https://github.com/trustgraph/trustgraph-holochain/tags).

HDK version correspondence:

- TrustGraph::Holochain `v0.0.1` - `v0.0.6` works with `hdk` version `0.0.116`
- TrustGraph::Holochain `v0.0.7` works with `hdk` version `0.0.125`
- TrustGraph::Holochain `v0.0.8` (pulled)
- TrustGraph::Holochain `v0.0.9` works with `hdk` version `0.0.131`
- TrustGraph::Holochain `v0.1.0` works with `hdk` version `0.1.1` and `hdi` version `0.2.1`

## Usage

### TrustAtom Creation

```rs
pub struct TrustAtomInput {
  pub target: AnyLinkableHash,
  pub content: Option<String>,
  pub value: Option<String>,
  pub extra: Option<BTreeMap<String, String>>,
}

#[hdk_extern]
pub fn create_trust_atom(input: TrustAtomInput) -> ExternResult<TrustAtom> {
    // ...
}
```

### TrustAtom Query

```rs
pub struct QueryInput {
  pub source: Option<AnyLinkableHash>,
  pub target: Option<AnyLinkableHash>,
  pub content_full: Option<String>,
  pub content_starts_with: Option<String>,
  pub value_starts_with: Option<String>,
}

#[hdk_extern]
pub fn query(input: QueryInput) -> ExternResult<Vec<TrustAtom>> {
    // ...
}
```

### TrustAtom

Client-facing representation of a Trust Atom (this is what is returned to client from a `query`)

```rs
pub struct TrustAtom {
  pub source_hash: AnyLinkableHash,
  pub target_hash: AnyLinkableHash,
  pub content: Option<String>,
  pub value: Option<String>,
  pub extra: Option<BTreeMap<String, String>>,
}
```

For more detailed usage, see also the tests: https://github.com/trustgraph/trustgraph-holochain/blob/main/zomes/trust_atom/tests/trust_atom_tests.rs

## Data format

We encode TrustAtoms as links, with the following components:

1. Holochain Link `base` == TrustAtom `source` - creating agent (`AgentPubKey`)
1. Holochain Link `target` == TrustAtom `target` - entity being rated/reviewed/etc - `AnyLinkableHash`
1. Holochain Link `tag`\* (max 999 bytes) - formatted as UTF-8 string

- TrustAtom header bytes: `[0xC5][0xA6]` (which together comprise the unicode character `Ŧ`) (required)
- Direction byte:
  - `[0x21][0x92]` (unicode `→`) means: HC target = TA target
  - `[0x21][0xA9]` (unicode `↩`) means: HC target = TA source
- TrustAtom `content` - semantic info (eg sushi) - max 900 bytes
- Separator: null byte `[0x00]`
- TrustAtom `value` - rating ( `"-0.999999999"` to `"0.999999999"`) - max 12 chars
- Separator: null byte `[0x00]`
- Random 9 characters for bucketing purposes
- Separator: null byte `[0x00]`
- Optional "extra" `EntryHash` if additional metadata is needed:
  - Entry contains attributes formatted in: `BTreeMap<String, String>`
  - Entry hash is a sring version of `EntryHash` (eg `uhCEkto7…`) for debugging purposes, not raw bytes

\*This format is designed to allow us to encode trust atoms as Holochain links, and search them by their tags. Holochain can search for all links _starting_ with a given set of bytes (characters).

### Full Example Link Tags

```
Ŧ→[0x00]sushi[0x00]0.999999999[0x00]892412523[0x00]uhCEk…UFnFF
Ŧ↩[0x00]sushi[0x00]0.999999999[0x00]892412523[0x00]uhCEk…UFnFF

Ŧ→[0x00]content[0x00]0.800000000[0x00]087423432[0x00]uhCEk…qS5wc
Ŧ↩[0x00]content[0x00]0.800000000[0x00]087423432[0x00]uhCEk…qS5wc

Ŧ→[0x00]spam[0x00]-0.999999999[0x00]328425615[0x00]uhCEk…VaaDd
Ŧ→[0x00]block[0x00]-0.999999999[0x00]837592944[0x00]uhCEk…VaaDd
```

## Roadmap

- [x] Create TrustAtoms as paired Holochain links
- [x] Fetch TrustAtoms by content leading bytes
- [x] Fetch TrustAtoms by content and value
- [ ] Integration into holochain example projects, eg [Clutter](https://github.com/artbrock/clutter)
- [ ] Roll up a TrustGraph by crawling TrustAtoms (2 levels deep)

## Authors

👤 **Harlan T Wood (https://github.com/harlantwood)**
👤 **Zeek (https://github.com/dauphin3)**

- Website: https://trustgraph.net
- Github: [@trustgraph](https://github.com/trustgraph)

## 🤝 Contributing

Contributions, issues and feature requests are welcome!<br />

<a href="https://github.com/trustgraph/trustgraph-holochain/issues/new?assignees=&labels=bug&template=01_BUG_REPORT.md&title=bug%3A+">Report a Bug</a>
·
<a href="https://github.com/trustgraph/trustgraph-holochain/issues/new?assignees=&labels=enhancement&template=02_FEATURE_REQUEST.md&title=feat%3A+">Request a Feature</a>
·
<a href="https://github.com/trustgraph/trustgraph-holochain/discussions">Ask a Question</a>

## Show your support

Give a ⭐️ if you like the project!

## Dev

If you're new to holochain dev, start here: <https://developer.holochain.org/quick-start>. Then, from a terminal in the root of this repo:

```
nix develop
```

Then within nix shell:

```
bin/run test             # rust tests
bin/run test_watch       # rust tests with watch

bin/run clippy           # rust linter
bin/run clippy_watch     # rust linter with watch

bin/run clean            # reset to clean repo state; removes all gitignored files
```

Or to run all checks:

```
bin/run checks
```

When you have commits that you are ready to push, to run checks and push only if checks are all green:

```
bin/run shipit
```

## 📝 License

Copyright © 2022 [Harlan T Wood (https://github.com/harlantwood)](https://github.com/trustgraph).<br />
This project is [Apache-2.0](https://github.com/trustgraph/js-trustgraph-core/blob/master/LICENSE) licensed.
