# tiny media server

is for serving your small-to-medium sized video library as a web page. In particularly useful if you want to stream movie from you desktop to TV/Projector/etc.

Goals:

- single binary for deploy, no need to configure/db setup/etc;
- lightweight, fast, and simple;
- ubiquitous support (not this time IE6 sorry, but IE9 should work).

Not goals:

- serving on public websites (no, nginx won't help you much);
- fancy UI (but contributions are welcome).

## Installing

- grab pre-built binary at [Release page](https://github.com/l4l/tiny-media-server/releases);
- or `cargo install tiny-media-server` to manually build it yourself.

## Development

Contributions are welcome but make sure you have discussed the feature before implementing it if it's big enough and/or controversial.

Make sure `cargo-fmt` and `cargo-clippy` are passing.
