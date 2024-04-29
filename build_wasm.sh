EMCC_CFLAGS="-sUSE_GLFW=3 -sGL_ENABLE_GET_PROC_ADDRESS -sASYNCIFY -sASSERTIONS" cargo build --release --target=wasm32-unknown-emscripten
