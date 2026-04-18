FROM rust:1.95.0
WORKDIR /app
COPY . .
RUN cargo build --release
EXPOSE 8080
CMD ["./target/release/cloud-rust"]
