# Windows Cross-Compilation CI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add CI support for cross-compiling `rmcp-openapi-server` to a Windows `.exe` using `cargo-xwin` on Linux.

**Architecture:** A new Docker image containing the Rust MSVC cross-compilation toolchain (`cargo-xwin`, clang, lld) is built/pushed via CI. A new `build:x86_64-pc-windows-msvc` job uses this image to cross-compile the workspace and produce an `.exe` artifact.

**Tech Stack:** Rust 1.94.0, cargo-xwin, clang-cl, Docker, GitLab CI

---

## File Structure

| Action | Path | Responsibility |
|--------|------|----------------|
| Create | `images/rust/x86_64-pc-windows-msvc/Dockerfile` | Cross-compilation Docker image |
| Modify | `.gitlab/ci/images.gitlab-ci.yml` | Image build/push CI jobs |
| Modify | `.gitlab/ci/rust.gitlab-ci.yml` | Windows build CI job |

---

### Task 1: Create the Windows MSVC Docker image

**Files:**
- Create: `images/rust/x86_64-pc-windows-msvc/Dockerfile`

- [ ] **Step 1: Create the Dockerfile**

```dockerfile
# Base image with Rust toolchain (Debian-based with glibc)
# renovate: datasource=docker depName=rust versioning=semver
FROM rust:1.94.0@sha256:f17e723020f87c1b4ac4ff6d73c9dfbb7d5cb978754c76641e47337d65f61e12

# Install clang, lld, and llvm for clang-cl backend (llvm provides llvm-lib)
RUN apt-get update && apt-get install -y --no-install-recommends \
    clang \
    lld \
    llvm \
    && rm -rf /var/lib/apt/lists/*

# Add Windows MSVC target and common Rust components
RUN rustup target add x86_64-pc-windows-msvc \
    && rustup component add rustfmt clippy

# Install cargo-xwin (musl binary)
# renovate: datasource=github-releases depName=rust-cross/cargo-xwin
ARG CARGO_XWIN_VERSION="0.21.4"
RUN wget -qO- "https://github.com/rust-cross/cargo-xwin/releases/download/v${CARGO_XWIN_VERSION}/cargo-xwin-v${CARGO_XWIN_VERSION}.x86_64-unknown-linux-musl.tar.gz" \
    | tar zxf - -C /usr/local/cargo/bin

# Install cargo-nextest
# renovate: datasource=crate depName=cargo-nextest
ARG NEXTEST_VERSION="0.9.122"
RUN wget -qO- "https://get.nexte.st/${NEXTEST_VERSION}/linux" | tar zxf - -C /usr/local/cargo/bin

# Install sccache for build caching (using musl binary)
# renovate: datasource=github-releases depName=mozilla/sccache
ARG SCCACHE_VERSION="0.14.0"
RUN wget -qO- "https://github.com/mozilla/sccache/releases/download/v${SCCACHE_VERSION}/sccache-v${SCCACHE_VERSION}-x86_64-unknown-linux-musl.tar.gz" | tar zxf - -C /tmp \
    && mv /tmp/sccache-v${SCCACHE_VERSION}-x86_64-unknown-linux-musl/sccache /usr/local/cargo/bin/ \
    && rm -rf /tmp/sccache-v${SCCACHE_VERSION}-x86_64-unknown-linux-musl

# Install just (using musl binary)
# renovate: datasource=github-releases depName=casey/just
ARG JUST_VERSION="1.47.1"
RUN wget -qO- "https://github.com/casey/just/releases/download/${JUST_VERSION}/just-${JUST_VERSION}-x86_64-unknown-linux-musl.tar.gz" | tar zxf - -C /usr/local/cargo/bin just

# Pre-cache Windows SDK and CRT
RUN cargo xwin cache xwin

# Set default target
ENV CARGO_BUILD_TARGET=x86_64-pc-windows-msvc
```

Write this to `images/rust/x86_64-pc-windows-msvc/Dockerfile`.

- [ ] **Step 2: Verify the Dockerfile structure**

Run: `head -1 images/rust/x86_64-pc-windows-msvc/Dockerfile`
Expected: `# Base image with Rust toolchain (Debian-based with glibc)`

- [ ] **Step 3: Commit**

```bash
git add images/rust/x86_64-pc-windows-msvc/Dockerfile
git commit -m "feat: add Docker image for Windows MSVC cross-compilation"
```

---

### Task 2: Add CI image build/push jobs

**Files:**
- Modify: `.gitlab/ci/images.gitlab-ci.yml`

The existing file ends at line 59 with the `push:rust-x86_64-unknown-linux-gnu` job. Append the two new jobs after it, following the same pattern.

- [ ] **Step 1: Add the image build job**

Append to `.gitlab/ci/images.gitlab-ci.yml`:

```yaml

# Build rust-x86_64-pc-windows-msvc image
image:rust-x86_64-pc-windows-msvc:
  extends: .docker-in-docker
  needs: []
  variables:
    DOCKERFILE_DIR: "images/rust/x86_64-pc-windows-msvc"
    OCI_IMAGE_NAME: "images/rust"
    OCI_IMAGE_VARIANT: "x86_64-pc-windows-msvc"
    GIT_FETCH_EXTRA_FLAGS: "+refs/heads/$CI_DEFAULT_BRANCH:refs/remotes/origin/$CI_DEFAULT_BRANCH"
  artifacts:
    expire_in: "1 hr"
    paths:
      - "./image.tar"
  rules:
    - if: '$IMAGE_BUILD_FORCE == "all" || $IMAGE_BUILD_FORCE == "rust-x86_64-pc-windows-msvc"'
    - if: '$IMAGE_PUSH_FORCE == "all" || $IMAGE_PUSH_FORCE == "rust-x86_64-pc-windows-msvc"'
    - changes:
        compare_to: "refs/heads/$CI_DEFAULT_BRANCH"
        paths:
          - "images/rust/x86_64-pc-windows-msvc/Dockerfile"
          - ".gitlab/ci/images.gitlab-ci.yml"
      if: '$CI_PIPELINE_SOURCE == "merge_request_event"'
    - changes:
        paths:
          - "images/rust/x86_64-pc-windows-msvc/Dockerfile"
          - ".gitlab/ci/images.gitlab-ci.yml"
      if: '$CI_PIPELINE_SOURCE == "push" && $CI_COMMIT_REF_NAME == $CI_DEFAULT_BRANCH'
  script: |
    IMAGE_TAG=$(grep "^FROM rust:" ${DOCKERFILE_DIR}/Dockerfile | sed -r "s/[^:]*:([0-9]+\.[0-9]+\.[0-9]+).*/\1/")
    OCI_PATH="$CI_REGISTRY_IMAGE/${OCI_IMAGE_NAME}:${IMAGE_TAG}-${OCI_IMAGE_VARIANT}"
    docker login -u gitlab-ci-token -p $CI_JOB_TOKEN $CI_REGISTRY
    docker build -t $OCI_PATH $DOCKERFILE_DIR
    docker save $OCI_PATH -o image.tar
```

- [ ] **Step 2: Add the image push job**

Continue appending to `.gitlab/ci/images.gitlab-ci.yml`:

```yaml

# Push rust-x86_64-pc-windows-msvc image
push:rust-x86_64-pc-windows-msvc:
  extends: .docker-in-docker
  needs:
    - image:rust-x86_64-pc-windows-msvc
  variables:
    DOCKERFILE_DIR: "images/rust/x86_64-pc-windows-msvc"
    OCI_IMAGE_NAME: "images/rust"
    OCI_IMAGE_VARIANT: "x86_64-pc-windows-msvc"
  rules:
    - if: '$IMAGE_PUSH_FORCE == "all" || $IMAGE_PUSH_FORCE == "rust-x86_64-pc-windows-msvc"'
    - changes:
        paths:
          - "images/rust/x86_64-pc-windows-msvc/Dockerfile"
          - ".gitlab/ci/images.gitlab-ci.yml"
      if: '$CI_PIPELINE_SOURCE == "push" && $CI_COMMIT_REF_NAME == $CI_DEFAULT_BRANCH'
  script: |
    IMAGE_TAG=$(grep "^FROM rust:" ${DOCKERFILE_DIR}/Dockerfile | sed -r "s/[^:]*:([0-9]+\.[0-9]+\.[0-9]+).*/\1/")
    OCI_PATH="$CI_REGISTRY_IMAGE/${OCI_IMAGE_NAME}:${IMAGE_TAG}-${OCI_IMAGE_VARIANT}"
    docker load -i image.tar
    docker login -u gitlab-ci-token -p $CI_JOB_TOKEN $CI_REGISTRY
    docker push $OCI_PATH
```

- [ ] **Step 3: Validate YAML syntax**

Run: `python3 -c "import yaml; yaml.safe_load(open('.gitlab/ci/images.gitlab-ci.yml'))"`
Expected: No errors (exit code 0). Note: YAML anchors from other files won't resolve, but syntax should be valid.

- [ ] **Step 4: Commit**

```bash
git add .gitlab/ci/images.gitlab-ci.yml
git commit -m "ci: add image build/push jobs for Windows MSVC cross-compilation"
```

---

### Task 3: Add the Windows build job

**Files:**
- Modify: `.gitlab/ci/rust.gitlab-ci.yml`

Add the new build job after the existing `build` job (after line 56).

- [ ] **Step 1: Add the Windows build job**

Insert after the existing `build` job in `.gitlab/ci/rust.gitlab-ci.yml`:

```yaml

# Windows cross-compilation build job
build:x86_64-pc-windows-msvc:
  image: registry.gitlab.com/lx-industries/rmcp-openapi/images/rust:1.94.0-x86_64-pc-windows-msvc
  extends:
    - .sccache-rust
  stage: build
  variables:
    CARGO_HOME: ".cargo"
  rules:
    - if: '$CI_PIPELINE_SOURCE == "merge_request_event"'
      changes: *rust-changes
    - if: "$CI_COMMIT_BRANCH == $CI_DEFAULT_BRANCH"
      changes: *rust-changes
  before_script:
    - !reference [.sccache-rust, before_script]
    - rustc --version
    - cargo --version
  script:
    - cargo xwin build --workspace --all-features --verbose
    - sccache --show-stats || true
  cache:
    - <<: *cargo-registry-cache
  artifacts:
    when: on_success
    expire_in: 1 week
    paths:
      - target/x86_64-pc-windows-msvc/debug/rmcp-openapi-server.exe
  interruptible: true
```

Note: The image reference does not include a digest pin yet. The digest will be known after the image is first built and pushed to the registry. Once pushed, update the image line to include `@sha256:<digest>`.

- [ ] **Step 2: Validate YAML syntax**

Run: `python3 -c "import yaml; yaml.safe_load(open('.gitlab/ci/rust.gitlab-ci.yml'))"`
Expected: Will fail due to YAML anchors (`*rust-changes`, `*cargo-registry-cache`) — this is expected since they're defined as anchors in the same file. Instead verify structure:

Run: `grep -c "build:x86_64-pc-windows-msvc" .gitlab/ci/rust.gitlab-ci.yml`
Expected: `1`

Run: `grep "cargo xwin build" .gitlab/ci/rust.gitlab-ci.yml`
Expected: `    - cargo xwin build --workspace --all-features --verbose`

- [ ] **Step 3: Commit**

```bash
git add .gitlab/ci/rust.gitlab-ci.yml
git commit -m "ci: add Windows MSVC cross-compilation build job"
```

---

### Task 4: Build and push the Docker image

This task must be done manually or via a CI pipeline trigger, since the Docker image needs to be built and pushed to the registry before the build job can use it.

- [ ] **Step 1: Trigger the image build**

Option A — Push to main and let CI build it automatically.

Option B — Trigger manually via GitLab CI with variable:
```
IMAGE_BUILD_FORCE=rust-x86_64-pc-windows-msvc
IMAGE_PUSH_FORCE=rust-x86_64-pc-windows-msvc
```

- [ ] **Step 2: Get the image digest**

After the image is pushed, get the digest from the GitLab Container Registry or from the push job logs. Look for:
```
1.94.0-x86_64-pc-windows-msvc: digest: sha256:<digest>
```

- [ ] **Step 3: Pin the image digest in the build job**

Update the image line in `.gitlab/ci/rust.gitlab-ci.yml`:

```yaml
  image: registry.gitlab.com/lx-industries/rmcp-openapi/images/rust:1.94.0-x86_64-pc-windows-msvc@sha256:<actual-digest>
```

- [ ] **Step 4: Commit the digest pin**

```bash
git add .gitlab/ci/rust.gitlab-ci.yml
git commit -m "ci: pin Windows MSVC image digest"
```
