FROM rust:1.60 AS builder

WORKDIR /effort_visualizer
COPY ./effort_visualizer/Cargo.toml Cargo.toml
RUN mkdir src
RUN echo "fn main(){}" > src/main.rs
RUN cargo build --release
COPY ./effort_visualizer/src ./src
RUN rm -f target/release/deps/effort_visualizer*
RUN cargo build --release

FROM rust:1.60

COPY --from=builder /effort_visualizer/target/release/effort_visualizer /usr/local/bin/effort_visualizer
CMD ["effort_visualizer"]