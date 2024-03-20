FROM rust:slim-bookworm AS build
WORKDIR /work

RUN apt-get update \
    && apt-get upgrade -y \
    && apt-get install -y --no-install-recommends protobuf-compiler

COPY . .

RUN cargo build -p peanutbutter --release --locked

FROM debian:bookworm-slim

ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser

COPY --from=build /work/target/release/peanutbutter /bin/

EXPOSE 50051

CMD ["/bin/peanutbutter"]
