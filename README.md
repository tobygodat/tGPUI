# tGPUI

Keystone's pinned fork of [GPUI](https://gpui.rs), Zed's GPU-accelerated UI framework.

## Upstream

The base is the published `gpui-pre` 0.3.9 crates, which are Zed's GPUI crates at
[zed-industries/zed@93d745668805029476e6d7fbb19652116fed51de](https://github.com/zed-industries/zed/commit/93d745668805029476e6d7fbb19652116fed51de),
renamed with a `gpui-pre-` prefix by the gpui-pre publisher. `crates/taffy` is the published
[taffy](https://github.com/DioxusLabs/taffy) 0.13.0.

The first commit imports those sources unmodified. Every later commit is one change the fork
carries, so `git log` lists the patches and `git diff <import commit>` shows them.

## Patches

- `gpui-pre-linux`: Wayland key and modifiers events that arrive before a keymap are dropped
  instead of panicking. Keystone KS-009, upstream zed-industries/zed#64660.
- `gpui-pre-linux`: Wayland displays report the logical position and size that xdg-output
  gives, so they are right under fractional scale and rotation and sit where the compositor puts
  them. Without xdg-output they are the current mode divided by the integer `wl_output` scale,
  as upstream. Keystone TOB-182.
- `taffy`: measurements evicted from the nine per-node cache slots are kept in an overflow list.
  Without it, nested flex layout re-measures subtrees on every pass and grows about fourfold per
  nesting level. An evicted measurement answers only a request with identical inputs.

## Using it

Pin an exact commit from a consumer's `Cargo.toml`:

```toml
gpui = { package = "gpui-pre", git = "https://github.com/tobygodat/tGPUI", rev = "<commit>" }
gpui_platform = { package = "gpui-pre-platform", git = "https://github.com/tobygodat/tGPUI", rev = "<commit>", features = ["wayland", "x11"] }
```

The crate names stay `gpui-pre-*` so the upstream diff stays readable and a consumer migrates
from gpui-pre by changing only the dependency source.

## Rebasing

To move to a newer snapshot, import the new `gpui-pre` crates as a fresh base commit and
cherry-pick the patch commits onto it. Drop any patch that upstream has taken.

## License

The GPUI crates are Apache-2.0, as upstream. Each keeps its own `LICENSE-APACHE`. See `LICENSE`.

`crates/taffy` is MIT, as upstream taffy. See `crates/taffy/LICENSE`.
