# Crossfeed for Popamp

A [Popamp](https://github.com/popamp/popamp) dsp plug-in for headphones.
In a room each ear hears both speakers, the far one a little later,
quieter and duller; on headphones each ear hears one channel only, so
a hard-panned instrument sounds as if it were inside your head. The
plug-in mixes each channel, low-passed, into the other, a few dB below
the direct sound. A sound in the middle keeps its level.

It is also the example of a dsp plug-in: `src/crossfeed.rs` is the
sound, tested natively, and `src/lib.rs` the two calls, `reset` and
`process`, described in
[popamp-plugin-api](https://github.com/popamp/popamp-plugin-api). It
needs no permissions.

## Install

You need Popamp built with plug-ins (`cargo build --release --features
plugins`).

```sh
git clone https://github.com/popamp/plugin-crossfeed ~/.local/share/popamp/plugins/crossfeed
```

The folder must be called `crossfeed`, the plug-in's id. Then in Popamp:
Options > Plug-ins…, tick Crossfeed and press OK. It works on everything
you play, after the equalizer.

To update: `git pull` in that folder.

## Settings

In Options > Plug-ins…:

- **Level**: how many dB below the direct sound the other channel comes
  in, 1 to 15; 4.5 by default. Lower is stronger.
- **Cutoff**: where the other channel starts to lose its highs, 300 to
  2000 Hz; 700 by default.

A setting out of range leaves the sound unchanged and soon switches the
plug-in off; Popamp's log says why. Fix the setting and tick it again.

## Build

```sh
rustup target add wasm32-unknown-unknown   # once
./build.sh                                 # writes plugin.wasm
cargo test                                 # the sound, natively
```

`plugin.wasm` is committed, so cloning the repository installs a working
plug-in; rebuild it with `./build.sh` whenever `src/` changes.

## License

MIT or Apache-2.0, at your option.
