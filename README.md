# preco

[![image](https://img.shields.io/pypi/v/preco.svg)](https://pypi.python.org/pypi/preco)
[![image](https://img.shields.io/pypi/l/preco.svg)](https://pypi.python.org/pypi/preco)
[![image](https://img.shields.io/pypi/pyversions/preco.svg)](https://pypi.python.org/pypi/preco)
[![Actions status](https://github.com/akx/preco/workflows/CI/badge.svg)](https://github.com/akx/preco/actions)

A partial reimplementation of [`pre-commit`](https://github.com/pre-commit/pre-commit) in Rust.

> [!IMPORTANT]
> Heavily just a proof-of-concept and work-in-progress.
> There are bits that could probably be trivially optimized and parallelized,
> but that's not happening yet.
>
> Will run your Python and Node hooks on `--all-files`, but not much more.
>
> Presently requires [uv](https://github.com/astral-sh/uv) to be available
> for virtualenv creation (and pnpm for Node deps), has only been tested on a Mac, etc.

## Acknowledgements

- This project is naturally heavily inspired by the original `pre-commit` project,
  and borrows a lot of its ideas and the configuration formats (for compatibility).
- As noted in the source for `crates/identify`, the filename and extension mappings
  are adapted from the [`pre-commit/identify`](https://github.com/pre-commit/identify)
  library.
- The basic Rust workspace, CLI and tracing was adapted from [uv](https://github.com/astral-sh/uv).

## License

Preco is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Preco by you, as defined in the Apache-2.0 license, shall be
dually licensed as above, without any additional terms or conditions.
