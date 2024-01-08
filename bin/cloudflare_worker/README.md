# Conductor (WASM) for CloudFlare Workers

## Running Locally

1. Ensure you have NodeJS (>=18), `pnpm` (8).
2. Install NodeJS dependencies by using: `pnpm install`
3. Run `pnpm dev`

> The default `dev` script is using the `test_config/worker.yaml` configuration from the root of this repository.

## Build

### Note on building for MacOS users

If you ran `cargo install -q worker-build && worker-build --release`, and faced something similar to the following error on macos:

```sh
  ...
  exit status: 1
  cargo:warning=ToolExecError: Command "clang" "-O3" "-ffunction-sections" "-fdata-sections" "-fPIC" "--target=wasm32-unknown-unknown" "-I" "wasm-shim/" "-I" "zstd/lib/" "-I" "zstd/lib/common" "-fvisibility=hidden" "-DXXH_STATIC_ASSERT=0" "-DZSTD_LIB_DEPRECATED=0" "-DXXH_PRIVATE_API=" "-DZSTDLIB_VISIBILITY=" "-DZSTDERRORLIB_VISIBILITY=" "-o" "/.../target/wasm32-unknown-unknown/release/build/zstd-sys-082f2962e2331fc1/out/zstd/lib/compress/zstd_ldm.o" "-c" "zstd/lib/compress/zstd_ldm.c" with args "clang" did not execute successfully (status code exit status: 1).
  exit status: 1
Error: wasm-pack exited with status exit status: 1
```

Then, follow the below steps:

1. Install LLVM: `brew install llvm`
1. Configure your env to use the following vars:

```sh
export LDFLAGS="-L/opt/homebrew/opt/llvm/lib"
export CPPFLAGS="-I/opt/homebrew/opt/llvm/include"
export PATH="/opt/homebrew/opt/llvm/bin:$PATH"
```
