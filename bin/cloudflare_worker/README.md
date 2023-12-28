## Note on building for macos users

If you ran `cargo install -q worker-build && worker-build --release`, and faced something similar to the following error on macos:
```sh
  ...
  exit status: 1
  cargo:warning=ToolExecError: Command "clang" "-O3" "-ffunction-sections" "-fdata-sections" "-fPIC" "--target=wasm32-unknown-unknown" "-I" "wasm-shim/" "-I" "zstd/lib/" "-I" "zstd/lib/common" "-fvisibility=hidden" "-DXXH_STATIC_ASSERT=0" "-DZSTD_LIB_DEPRECATED=0" "-DXXH_PRIVATE_API=" "-DZSTDLIB_VISIBILITY=" "-DZSTDERRORLIB_VISIBILITY=" "-o" "/Users/yassineldeeb/development/conductor-t2/target/wasm32-unknown-unknown/release/build/zstd-sys-082f2962e2331fc1/out/zstd/lib/compress/zstd_ldm.o" "-c" "zstd/lib/compress/zstd_ldm.c" with args "clang" did not execute successfully (status code exit status: 1).
  exit status: 1
  cargo:warning=ToolExecError: Command "clang" "-O3" "-ffunction-sections" "-fdata-sections" "-fPIC" "--target=wasm32-unknown-unknown" "-I" "wasm-shim/" "-I" "zstd/lib/" "-I" "zstd/lib/common" "-fvisibility=hidden" "-DXXH_STATIC_ASSERT=0" "-DZSTD_LIB_DEPRECATED=0" "-DXXH_PRIVATE_API=" "-DZSTDLIB_VISIBILITY=" "-DZSTDERRORLIB_VISIBILITY=" "-o" "/Users/yassineldeeb/development/conductor-t2/target/wasm32-unknown-unknown/release/build/zstd-sys-082f2962e2331fc1/out/zstd/lib/compress/zstd_opt.o" "-c" "zstd/lib/compress/zstd_opt.c" with args "clang" did not execute successfully (status code exit status: 1).
  exit status: 1
  cargo:warning=ToolExecError: Command "clang" "-O3" "-ffunction-sections" "-fdata-sections" "-fPIC" "--target=wasm32-unknown-unknown" "-I" "wasm-shim/" "-I" "zstd/lib/" "-I" "zstd/lib/common" "-fvisibility=hidden" "-DXXH_STATIC_ASSERT=0" "-DZSTD_LIB_DEPRECATED=0" "-DXXH_PRIVATE_API=" "-DZSTDLIB_VISIBILITY=" "-DZSTDERRORLIB_VISIBILITY=" "-o" "/Users/yassineldeeb/development/conductor-t2/target/wasm32-unknown-unknown/release/build/zstd-sys-082f2962e2331fc1/out/zstd/lib/compress/zstdmt_compress.o" "-c" "zstd/lib/compress/zstdmt_compress.c" with args "clang" did not execute successfully (status code exit status: 1).
  exit status: 1
  cargo:warning=ToolExecError: Command "clang" "-O3" "-ffunction-sections" "-fdata-sections" "-fPIC" "--target=wasm32-unknown-unknown" "-I" "wasm-shim/" "-I" "zstd/lib/" "-I" "zstd/lib/common" "-fvisibility=hidden" "-DXXH_STATIC_ASSERT=0" "-DZSTD_LIB_DEPRECATED=0" "-DXXH_PRIVATE_API=" "-DZSTDLIB_VISIBILITY=" "-DZSTDERRORLIB_VISIBILITY=" "-o" "/Users/yassineldeeb/development/conductor-t2/target/wasm32-unknown-unknown/release/build/zstd-sys-082f2962e2331fc1/out/zstd/lib/decompress/huf_decompress.o" "-c" "zstd/lib/decompress/huf_decompress.c" with args "clang" did not execute successfully (status code exit status: 1).
  exit status: 1
  cargo:warning=ToolExecError: Command "clang" "-O3" "-ffunction-sections" "-fdata-sections" "-fPIC" "--target=wasm32-unknown-unknown" "-I" "wasm-shim/" "-I" "zstd/lib/" "-I" "zstd/lib/common" "-fvisibility=hidden" "-DXXH_STATIC_ASSERT=0" "-DZSTD_LIB_DEPRECATED=0" "-DXXH_PRIVATE_API=" "-DZSTDLIB_VISIBILITY=" "-DZSTDERRORLIB_VISIBILITY=" "-o" "/Users/yassineldeeb/development/conductor-t2/target/wasm32-unknown-unknown/release/build/zstd-sys-082f2962e2331fc1/out/zstd/lib/decompress/zstd_ddict.o" "-c" "zstd/lib/decompress/zstd_ddict.c" with args "clang" did not execute successfully (status code exit status: 1).
  exit status: 1
  cargo:warning=ToolExecError: Command "clang" "-O3" "-ffunction-sections" "-fdata-sections" "-fPIC" "--target=wasm32-unknown-unknown" "-I" "wasm-shim/" "-I" "zstd/lib/" "-I" "zstd/lib/common" "-fvisibility=hidden" "-DXXH_STATIC_ASSERT=0" "-DZSTD_LIB_DEPRECATED=0" "-DXXH_PRIVATE_API=" "-DZSTDLIB_VISIBILITY=" "-DZSTDERRORLIB_VISIBILITY=" "-o" "/Users/yassineldeeb/development/conductor-t2/target/wasm32-unknown-unknown/release/build/zstd-sys-082f2962e2331fc1/out/zstd/lib/decompress/zstd_decompress.o" "-c" "zstd/lib/decompress/zstd_decompress.c" with args "clang" did not execute successfully (status code exit status: 1).
  exit status: 1
  cargo:warning=ToolExecError: Command "clang" "-O3" "-ffunction-sections" "-fdata-sections" "-fPIC" "--target=wasm32-unknown-unknown" "-I" "wasm-shim/" "-I" "zstd/lib/" "-I" "zstd/lib/common" "-fvisibility=hidden" "-DXXH_STATIC_ASSERT=0" "-DZSTD_LIB_DEPRECATED=0" "-DXXH_PRIVATE_API=" "-DZSTDLIB_VISIBILITY=" "-DZSTDERRORLIB_VISIBILITY=" "-o" "/Users/yassineldeeb/development/conductor-t2/target/wasm32-unknown-unknown/release/build/zstd-sys-082f2962e2331fc1/out/zstd/lib/decompress/zstd_decompress_block.o" "-c" "zstd/lib/decompress/zstd_decompress_block.c" with args "clang" did not execute successfully (status code exit status: 1).cargo:warning=clang -cc1as: error: unknown target triple 'wasm32-unknown-unknown', please use -triple or -arch


  exit status: 1
  cargo:warning=ToolExecError: Command "clang" "-O3" "-ffunction-sections" "-fdata-sections" "-fPIC" "--target=wasm32-unknown-unknown" "-I" "wasm-shim/" "-I" "zstd/lib/" "-I" "zstd/lib/common" "-fvisibility=hidden" "-DXXH_STATIC_ASSERT=0" "-DZSTD_LIB_DEPRECATED=0" "-DXXH_PRIVATE_API=" "-DZSTDLIB_VISIBILITY=" "-DZSTDERRORLIB_VISIBILITY=" "-o" "/Users/yassineldeeb/development/conductor-t2/target/wasm32-unknown-unknown/release/build/zstd-sys-082f2962e2331fc1/out/zstd/lib/decompress/huf_decompress_amd64.o" "-c" "zstd/lib/decompress/huf_decompress_amd64.S" with args "clang" did not execute successfully (status code exit status: 1).

  --- stderr


  error occurred: Command "clang" "-O3" "-ffunction-sections" "-fdata-sections" "-fPIC" "--target=wasm32-unknown-unknown" "-I" "wasm-shim/" "-I" "zstd/lib/" "-I" "zstd/lib/common" "-fvisibility=hidden" "-DXXH_STATIC_ASSERT=0" "-DZSTD_LIB_DEPRECATED=0" "-DXXH_PRIVATE_API=" "-DZSTDLIB_VISIBILITY=" "-DZSTDERRORLIB_VISIBILITY=" "-o" "/Users/yassineldeeb/development/conductor-t2/target/wasm32-unknown-unknown/release/build/zstd-sys-082f2962e2331fc1/out/zstd/lib/decompress/huf_decompress_amd64.o" "-c" "zstd/lib/decompress/huf_decompress_amd64.S" with args "clang" did not execute successfully (status code exit status: 1).


Error: Compiling your crate to WebAssembly failed
Caused by: Compiling your crate to WebAssembly failed
Caused by: failed to execute `cargo build`: exited with exit status: 101
  full command: cd "/Users/yassineldeeb/development/conductor-t2/bin/cloudflare_worker" && "cargo" "build" "--lib" "--release" "--target" "wasm32-unknown-unknown"
Error: wasm-pack exited with status exit status: 1
```

Then, follow the below steps:
```sh
  export LDFLAGS="-L/opt/homebrew/opt/llvm/lib"
  export CPPFLAGS="-I/opt/homebrew/opt/llvm/include"
  export PATH="/opt/homebrew/opt/llvm/bin:$PATH"
brew install llvm
```

And retry again, it should be resolved!