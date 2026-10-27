# dog online companion

a progress tracker for dog online, built with leptos and compiled to webassembly.

it has four tabs:

- **bravery**: enemies you've killed
- **coordination**: fish you've caught
- **hardiness**: items you've harvested
- **dailies**: a daily checklist that resets at midnight pacific time

your progress is saved in your browser's local storage.

## running locally

you need rust (the toolchain is pinned in `rust-toolchain.toml`) and [trunk](https://trunkrs.dev):

```sh
cargo install trunk
trunk serve
```
