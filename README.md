# Virtual Picross

The first Virtual Boy homebrew game written in Rust! This is a small Picross game, with puzzles representing every VB game ever officially released.

## Build

To build this project, you will need to install [v810-rust](https://github.com/SupernaviX/v810-rust) and `just`.

```sh
just build
```

If you use vscode, you can make it stop reporting test-only errors (there is no support for running tests against the v810 target) by adding this to `.vscode/settings.json`:
```json
{
    "rust-analyzer.check.allTargets": false
}
```

## Credits

Developer: Simon Gellis
Yoster Island font: codeman38 https://www.1001fonts.com/yoster-island-font.html
Megu sprites by Megu-tan, ripped by hewkii