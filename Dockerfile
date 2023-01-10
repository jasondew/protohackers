FROM rust:1.66 as builder

WORKDIR /usr/src/protohackers
COPY . .

RUN cargo install --path .

# Deployed container
FROM debian:buster-slim

COPY --from=builder /usr/local/cargo/bin/protohackers /usr/local/bin/protohackers

CMD ["protohackers"]
