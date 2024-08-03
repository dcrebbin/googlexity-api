# Use an official Rust image as a builder
FROM rust:1.75 as builder

# Create a new empty shell project
RUN USER=root cargo new --bin rust_actix_web_template
WORKDIR /rust_actix_web_template

# Copy the Cargo.toml and Cargo.lock files and build dependencies
COPY . .
RUN cargo build --release
RUN rm src/*.rs

# Copy the source code and build the application
COPY ./src ./src
RUN rm ./target/release/deps/rust_actix_web_template*
RUN cargo build --release

# Use the same Rust base image for the final image
FROM rust:1.75
COPY --from=builder /rust_actix_web_template/target/release/rust_actix_web_template /usr/local/bin/rust_actix_web_template
EXPOSE 443 80
CMD ["rust_actix_web_template"]

