# Windows Cross-Compilation CI

Add CI support for cross-compiling `rmcp-openapi-server` to Windows (x86_64-pc-windows-msvc) using `cargo-xwin` on Linux.

## Scope

- Build-only (no tests, no clippy, no release profile)
- Produces a debug `.exe` artifact
- Docker image for the cross-compilation toolchain
- CI jobs for building/pushing the image and running the Windows build

## Docker Image

**Path:** `images/rust/x86_64-pc-windows-msvc/Dockerfile`

**Base:** `rust:1.94.0` (Debian, pinned by digest)

**Toolchain:**
- `clang` + `lld` + `llvm` for clang-cl backend
- `rustup target add x86_64-pc-windows-msvc`
- `rustfmt` + `clippy` components

**Tools (musl static binaries):**
- `cargo-xwin` 0.21.4 — Windows SDK/CRT cross-compilation
- `cargo-nextest` 0.9.122 — test runner (included for future use)
- `sccache` 0.14.0 — build caching
- `just` 1.47.1 — task runner (included for future use)

**Pre-caching:** `cargo xwin cache xwin` pre-downloads Windows SDK and CRT headers at image build time.

**Default target:** `ENV CARGO_BUILD_TARGET=x86_64-pc-windows-msvc`

## CI Image Build/Push Jobs

Added to `.gitlab/ci/images.gitlab-ci.yml`, following the existing Linux image pattern.

### `image:rust-x86_64-pc-windows-msvc`

- Extends `.docker-in-docker`
- Builds the Docker image, saves as `image-windows.tar` artifact (1 hour expiry)
- Triggers on:
  - `IMAGE_BUILD_FORCE` or `IMAGE_PUSH_FORCE` matching `all` or `rust-x86_64-pc-windows-msvc`
  - Dockerfile changes in MRs (compared to default branch)
  - Dockerfile changes pushed to main

### `push:rust-x86_64-pc-windows-msvc`

- Extends `.docker-in-docker`
- Needs `image:rust-x86_64-pc-windows-msvc`
- Loads tar, pushes to registry
- Triggers on:
  - `IMAGE_PUSH_FORCE` matching `all` or `rust-x86_64-pc-windows-msvc`
  - Dockerfile changes pushed to main

**Registry path:** `registry.gitlab.com/lx-industries/rmcp-openapi/images/rust:1.94.0-x86_64-pc-windows-msvc`

## Windows Build Job

Added to `.gitlab/ci/rust.gitlab-ci.yml`.

### `build:x86_64-pc-windows-msvc`

- **Image:** The MSVC cross-compilation image (pinned by digest)
- **Stage:** `build` (parallel with existing Linux build)
- **Extends:** `.sccache-rust`
- **Command:** `cargo xwin build --workspace --all-features --verbose`
- **Cache:** Reuses existing `*cargo-registry-cache` anchor
- **Rules:** Same `*rust-changes` pattern as other Rust jobs
- **Artifact:** `target/x86_64-pc-windows-msvc/debug/rmcp-openapi-server.exe` (1 week expiry)
- **Interruptible:** yes
- **No dependencies on other jobs** — runs independently

## Files Changed

1. **New:** `images/rust/x86_64-pc-windows-msvc/Dockerfile`
2. **Modified:** `.gitlab/ci/images.gitlab-ci.yml` — add image build/push jobs
3. **Modified:** `.gitlab/ci/rust.gitlab-ci.yml` — add `build:x86_64-pc-windows-msvc` job
