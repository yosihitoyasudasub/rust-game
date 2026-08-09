# Bundled fonts

Both faces are compiled into the binary, so their licences travel with every build.
Both are SIL Open Font License 1.1, which permits embedding and redistribution.

| File | Source | Licence |
|---|---|---|
| `ZenKakuGothicNew-Regular.ttf` | [google/fonts — ofl/zenkakugothicnew](https://github.com/google/fonts/tree/main/ofl/zenkakugothicnew) | OFL 1.1 |
| `JetBrainsMono-Regular.ttf` | [JetBrains/JetBrainsMono](https://github.com/JetBrains/JetBrainsMono) | OFL 1.1 |

`jp-subset.ttf` and `mono-subset.ttf` are generated from those two by
`tools/subset-font` and are what the game actually embeds. Subsets of an OFL font
remain under the OFL.

Regenerate them after changing any on-screen text:

```
cargo run --release --manifest-path tools/subset-font/Cargo.toml
```

The two full source files are kept only as the subsetter's input; they are not
compiled in.
