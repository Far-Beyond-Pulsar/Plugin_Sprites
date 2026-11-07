# Plugin_Sprites

`pulsar.sprites`: 2D sprites drawn over the scene: their component, rows, image binding and the renderer feature that culls, batches and draws them.
Pulsar-Native vendors this repository at `plugins/vendor/sprites`; it builds only
inside a Pulsar-Native checkout (see `README.md`).

## Required for every agent

Before reading, designing, editing, reviewing or briefing any change here,
load the **`helio-architecture`** skill. Its single source is Helio's
[`.agents/skills/helio-architecture/SKILL.md`](https://github.com/Far-Beyond-Pulsar/Helio/blob/main/.agents/skills/helio-architecture/SKILL.md); in a Pulsar-Native
checkout it is at `crates/renderer/helio/.agents/skills/helio-architecture/SKILL.md`.
That applies to bug fixes, perf work, tests and small changes too.

Follow the skill's steps in order and end with its report section (Design,
AAA comparison, Deleted, Verified). Anything you brief (a subagent, another
session or another agent) must load the skill too. The skill is the single
source of these rules: this file only points to it, so change the skill
rather than copying rules here.

## Docs

| File | What |
|---|---|
| [Helio `docs/architecture/plugin-migration.md`](https://github.com/Far-Beyond-Pulsar/Helio/blob/main/docs/architecture/plugin-migration.md) | Every object kind is a plugin; the plugin table; "As built" notes per phase |
| [Helio `docs/architecture/layered-rewrite.md`](https://github.com/Far-Beyond-Pulsar/Helio/blob/main/docs/architecture/layered-rewrite.md) | Layers, rules, buffer contracts, extension points |
| Pulsar-Native `AGENTS.md`, `.agents/PLUGIN_SYSTEM.md`, `.agents/CRATES.md` | Plugin descriptors, the installed set, submodule pins |

## Key commands (from the Pulsar-Native root)

```
cargo check -p sprites_component -p sprites_plugin -p sprites-examples -p helio-pass-sprite-batch -p helio-pass-sprite-cull -p helio-pass-radiance-cascades-2d -p helio-pass-sprite-simulate --tests
cargo test -p sprites_component -p helio-pass-sprite-batch -p helio-pass-sprite-cull -p helio-pass-radiance-cascades-2d -p helio-pass-sprite-simulate
cargo test -p one_owner          every concept has one owner (scans this plugin)
cargo test -p installed_plugins  the installed set and absent plugins
```
