# dog online companion

a progress tracker for dog online, built with leptos and compiled to webassembly.

it has six tabs:

- **coordination**: fish you've caught
- **creativity**: books you've read
- **nutrition**: foods you've eaten
- **bravery**: enemies you've killed
- **hardiness**: items you've harvested
- **dailies**: a daily checklist that resets at midnight pacific time

your progress is saved in your browser's local storage.

## running locally

you need rust (the toolchain is pinned in `rust-toolchain.toml`) and [trunk](https://trunkrs.dev):

```sh
cargo install trunk
trunk serve
```

## checking against the wiki

to see where the app's data has drifted from the [dog online wiki](https://dogonline.wiki):

```sh
python3 -I scripts/check_wiki_data.py [--tab nutrition] [--gaps] [--json]
```

it prints only the differences (`+` only on the wiki, `-` only in the app, `~` a value differs) and exits 1 if there are any. `?` lines count items the wiki has no value for; `--gaps` lists them.

## attribution

the enemy, fish, book, item and food images in `assets/wiki` come from the [dog online wiki](https://dogonline.wiki) and its contributors, and are used under [cc by-sa 4.0](https://creativecommons.org/licenses/by-sa/4.0/) (full text in `assets/wiki/LICENSE`). `assets/wiki/manifest.json` links to the wiki page for each image.

the files are unmodified apart from being renamed. the app shows them in greyscale until you mark them complete.
