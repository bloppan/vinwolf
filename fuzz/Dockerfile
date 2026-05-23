FROM debian:bookworm-slim

WORKDIR /app

COPY --chmod=755 vinwolf-target-tiny /usr/local/bin/vinwolf-target-tiny
COPY --chmod=755 vinwolf-target-full /usr/local/bin/vinwolf-target-full
COPY --chmod=755 docker-entrypoint.sh /usr/local/bin/vinwolf-target-entrypoint

ENTRYPOINT ["/usr/local/bin/vinwolf-target-entrypoint"]
