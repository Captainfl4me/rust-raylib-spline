FROM ubuntu:latest as build

RUN apt-get update && apt-get upgrade -y && \
	apt-get install curl git python3 xz-utils build-essential cmake clang -y && \
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

RUN	rustup toolchain install nightly && \
	rustup default nightly && \
	rustup target add wasm32-unknown-emscripten

RUN git clone https://github.com/emscripten-core/emsdk.git && \
	cd ./emsdk && ./emsdk install latest && ./emsdk activate latest

RUN mkdir /app 

COPY ./Cargo.* /app/
COPY ./build_wasm.sh /app/
COPY ./src/ /app/src/
COPY ./assets/background_tile.png /app/assets/

WORKDIR /app/

RUN cd /emsdk && ls && . ./emsdk_env.sh && cd /app && ./build_wasm.sh

FROM nginx:alpine as run

COPY ./index.html /usr/share/nginx/html/
COPY --from=build /app/target/wasm32-unknown-emscripten/release/spline-drawer.* /usr/share/nginx/html/
COPY --from=build /app/target/wasm32-unknown-emscripten/release/spline_drawer.* /usr/share/nginx/html/
