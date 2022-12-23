# `workflow-jobs`

`workflow-jobs` reads GitHub Action workflow definitions and outputs the resulting GitHub job names in the exact same order they appear in the workflow definition.

## Examples

### `testdata/test.yml`

```
$ workflow-jobs testdata/test.yml 
enarx sev nightly debug
enarx sev nightly debug with dbg
enarx sev nightly release
enarx sgx nightly debug
enarx sgx nightly debug with dbg
enarx sgx nightly release
enarx kvm nightly debug
enarx kvm nightly debug with dbg
enarx kvm nightly release
enarx build-only nightly default-features
enarx build-only nightly gdb
enarx MacOS
enarx Windows
nightly debug
nightly release
sallyport miri debug
sallyport miri release
```

### `testdata/check.yml`

```
$ workflow-jobs testdata/check.yml 
nix fmt
checks (macos-latest, x86_64-darwin, clippy)
checks (macos-latest, x86_64-darwin, nextest)
checks (ubuntu-latest, x86_64-linux, clippy)
checks (ubuntu-latest, x86_64-linux, nextest)
checks (ubuntu-latest, x86_64-linux, fmt)
```

### List all job names in a repository

```
$ workflow-jobs .github/workflows/*
```

#### https://github.com/enarx/enarx/tree/6bbf266ba281cb695fffc589bb9e678cbb239928

```
$ workflow-jobs .github/workflows/*
create-pr
Conventional Commit Message Checker (Commisery)
sev coverage
sgx coverage
kvm coverage
nil coverage
dependabot
triage
cargo fmt
cargo clippy (--workspace --all-targets)
cargo clippy (--target=x86_64-unknown-linux-musl --workspace --all-targets)
cargo clippy (--target=x86_64-unknown-none -p enarx-shim-sgx -p enarx-shim-kvm -p sallyport -p enarx_syscall_tests)
cargo clippy (--target=wasm32-wasi -p enarx_wasm_tests --all-targets)
cargo deny
check-spdx-headers
nix-update
check
fmt
run
develop
auto-merge
build-nix (macos-latest, aarch64-apple-darwin, file ./result/bin/enarx, echo "OCI runtime not available, skip")
build-nix (ubuntu-latest, aarch64-unknown-linux-musl, nix shell --inputs-from . 'nixpkgs#qemu' -c qemu-aarch64 ./result/bin/enarx platform info, docker load < ./result)
build-nix (macos-latest, x86_64-apple-darwin, ./result/bin/enarx platform info, echo "OCI runtime not available, skip")
build-nix (ubuntu-latest, x86_64-unknown-linux-musl, ./result/bin/enarx platform info, docker load < ./result
docker run --rm enarx:$(nix eval --raw .#enarx-x86_64-unknown-linux-musl-oci.imageTag) enarx platform info
)
enarx Windows build
sign-x86_64
build-lipo
test-lipo (macos-latest)
test-lipo (aarch64-apple-darwin)
build-rpm (x86_64)
build-rpm (aarch64)
build-deb (x86_64, amd64)
build-deb (aarch64, arm64)
push_oci
release
create-pr
Run cargo-cyclonedx and generate BOM files [both JSON and XML]
Regenerate-BOM
test-docs docs/Install.md git,helloworld ubuntu
test-docs docs/Install.md git,helloworld debian
test-docs docs/Install.md git,helloworld fedora
test-docs docs/Install.md git,helloworld centos7
test-docs docs/Install.md git,helloworld centos8
enarx sev nightly debug
enarx sev nightly debug with dbg
enarx sev nightly release
enarx sgx nightly debug
enarx sgx nightly debug with dbg
enarx sgx nightly release
enarx kvm nightly debug
enarx kvm nightly debug with dbg
enarx kvm nightly release
enarx build-only nightly default-features
enarx build-only nightly gdb
enarx MacOS
enarx Windows
nightly debug
nightly release
sallyport miri debug
sallyport miri release
```
