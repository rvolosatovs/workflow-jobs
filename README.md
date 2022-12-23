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
