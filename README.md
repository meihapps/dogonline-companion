# dog online companion

a progress tracker for dog online, built with leptos and compiled to webassembly.

it has five tabs:

- **bravery**: enemies you've killed
- **coordination**: fish you've caught
- **hardiness**: items you've harvested
- **nutrition**: foods you've eaten
- **dailies**: a daily checklist that resets at midnight pacific time

your progress is saved in your browser's local storage.

## running locally

you need rust (the toolchain is pinned in `rust-toolchain.toml`) and [trunk](https://trunkrs.dev):

```sh
cargo install trunk
trunk serve
```

## attribution

the enemy, fish, item and food images in `assets/wiki` come from the [dog online wiki](https://dogonline.wiki) and its contributors, and are used under [cc by-sa 4.0](https://creativecommons.org/licenses/by-sa/4.0/) (full text in `assets/wiki/LICENSE`). `assets/wiki/manifest.json` links to the wiki page for each image.

the files are unmodified apart from being renamed. the app shows them in greyscale until you mark them complete.
