# Build the binary

```shell
cargo build --release
```

The binary will be in `target/release/unpack_moodle`

# Usage

The binary takes up to 4 arguments

To see the docs, run

```shell
unpack_moodle --help
```

## Arguments

- `--filter` specify a directory, which contains only `.txt` files
  - Each row will be read
  - `//` style comments are allowed
  - The program will search all zip files for the first match (no regex, just `.contains()`)
- `--source` the path to the input `.zip` file
- `--target` the name of the directory, where the output files will be placed
