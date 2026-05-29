# Release Checklist

This document is for maintainers publishing a RustGauge release.

## 1. Check the working tree

```powershell
git status
```

Commit or discard local changes before creating a release tag.

## 2. Verify the build

```powershell
cargo fmt --all -- --check
cargo check --locked
cargo clippy --locked -- -D warnings
cargo build --release
```

## 3. Create the release archive

```powershell
Compress-Archive -Force `
  -Path target\release\rust-gauge.exe,config\example.toml,README.md,README.ja.md,LICENSE-MIT,LICENSE-APACHE `
  -DestinationPath rust-gauge-windows-x64.zip
```

Do not commit the zip file or anything under `target/`.

## 4. Tag the release

Use the version from `Cargo.toml`.

```powershell
git tag v0.1.0
git push origin v0.1.0
```

## 5. Publish on GitHub

1. Open <https://github.com/pino-lab/rust-gauge/releases>.
2. Click `Draft a new release`.
3. Select the tag, for example `v0.1.0`.
4. Upload `rust-gauge-windows-x64.zip`.
5. Publish the release.

## 6. After publishing

Download the zip from the release page and run `rust-gauge.exe` once to confirm the published artifact works.
