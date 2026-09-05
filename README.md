# ytb

A git-style YouTube CLI: search once, then act on results by their number.

`ytb` turns `yt-dlp` + `mpv` into a fast terminal workflow. You search, get a
numbered list, and reference results by index — no REPL, no API key, no mouse.

```console
$ ytb search lofi hip hop
  1  3:32       lofi hip hop radio - beats to relax/study to           Lofi Girl
  2  1:01:01    Chillhop Essentials · Autumn 2020                      Chillhop Music
  3  -          synthwave radio 🌌 beats to chill/game to              Lofi Girl
  ...

$ ytb play 1          # open #1 in mpv
$ ytb listen 2 3 1    # audio-only queue, in that order
$ ytb url 2 | pbcopy  # copy the URL of #2
```

## Install

Requires [Rust](https://rustup.rs) to build, and these tools at runtime:

- **[yt-dlp](https://github.com/yt-dlp/yt-dlp)** — searching and downloading
- **[mpv](https://mpv.io)** — `play` / `listen`
- **[ffmpeg](https://ffmpeg.org)** — only for `mp3`

```console
brew install yt-dlp mpv ffmpeg   # macOS

git clone https://github.com/meomunm/ytb-youtube-cli
cd ytb-youtube-cli
cargo build --release
cp target/release/ytb /usr/local/bin/   # or anywhere on your PATH
```

Dependencies are checked lazily: `ytb list` works even without `mpv`
installed — you only need a tool when you invoke the command that uses it.

## Commands

| Command | Alias | Does |
|---|---|---|
| `search <query...>` | `s` | Find videos and save the numbered list |
| `list` | `ls`, `l` | Reprint the last results |
| `play <n...>` | `p` | Play in mpv (keeps a queue) |
| `listen <n...>` | `a` | Audio only (`mpv --no-video`) |
| `ascii <n...>` | `x` | Play as characters right in the terminal |
| `open <n...>` | `o` | Open in the browser (`open`) |
| `url [n...]` | `u` | Print watch URLs (no args → all) |
| `dl <n...>` | `d`, `download` | Download the video |
| `mp3 <n...>` | | Download and convert to mp3 |
| `help` | `-h`, `--help` | Show help |

Indices are 1-based. You can repeat and reorder them: `ytb play 3 1 1 2`
plays #3, then #1 twice, then #2 — all in a single mpv queue.

## Design notes

- **Validate everything before acting.** `ytb play 1 99 3` fails immediately
  (99 is out of range) and plays nothing — never a half-executed action.
- **Clean stdout.** Only data (the list, URLs) goes to stdout; progress and
  errors go to stderr, so `ytb url 1 | pbcopy` copies exactly one line.
- **One invocation, whole queue.** Every player command spawns its tool once
  with all the URLs, inheriting your terminal so mpv's keybindings work.
- **Empty results don't clobber.** A search that finds nothing leaves your
  previous list intact.

## Configuration

| Variable | Meaning |
|---|---|
| `YTB_COUNT` | Results per search (default `20`) |
| `YTB_ASCII_VO` | mpv video driver for `ascii` (default `tct`; e.g. `caca`, `sixel`) |
| `NO_COLOR` | Set to any value to disable colored output |

Results are cached at `${XDG_CACHE_HOME:-$HOME/.cache}/ytb/results.tsv`.

### Terminal video (`ascii`)

`ytb ascii 1` plays a video as text in your terminal — no window, just
characters. It uses mpv's built-in `tct` output (true-color character cells),
so it works out of the box and keeps the audio. Pick another driver with
`YTB_ASCII_VO`:

```console
ytb ascii 1                 # default: mpv --vo=tct
YTB_ASCII_VO=caca ytb ascii 1   # classic ASCII art (needs libcaca)
YTB_ASCII_VO=sixel ytb ascii 1  # sixel graphics (needs a sixel terminal)
```

Quality depends on your terminal size and font; it looks best in a true-color
terminal (iTerm2, kitty, WezTerm). Check which drivers your mpv supports with
`mpv --vo=help`.

## Development

```console
cargo test                     # unit tests
cargo clippy --all-targets     # lints
cargo fmt --check              # formatting
```

The crate is `std`-only — zero external dependencies.

## License

MIT — see [LICENSE](LICENSE).
