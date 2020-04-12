FROM rust:alpine AS builder

RUN apk add -U openssl-dev libc-dev

WORKDIR /usr/src/codename
COPY . .
RUN RUSTFLAGS="-C target-feature=-crt-static" cargo install --path .

FROM alpine:3.11

RUN apk add -U openssl libgcc

COPY --from=builder /usr/local/cargo/bin/codename /usr/bin/codename
RUN mkdir /maps

CMD ["codename", "/maps/maps.json"]
