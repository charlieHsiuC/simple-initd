# Simple-initd
Simple Init deamon

## Development

Format check:

```bash
cargo fmt-check
```

Run lint checks with Clippy:

```bash
cargo lint
```

## Buildroot

1. Install simple-initd to Buildroot:

```bash
./tools/buildroot/simple-initd/install_buildroot.sh <buildroot_dir>
```

2. Download dependencies:

```bash
cargo vendor
```

3. Enable simple-initd in Buildroot:

```bash
make menuconfig
```

Select `System tools` -> `simple-initd` and save the configuration.