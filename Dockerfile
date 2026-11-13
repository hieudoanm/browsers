FROM debian:trixie-slim

ARG PORT=8080

RUN apt-get update && apt-get install -y --no-install-recommends \
    libfontconfig1 \
    libfreetype6 \
    libgl1 \
    libegl1 \
    libx11-6 \
    libxcb-shape0 \
    libxcb-xfixes0 \
    libglib2.0-0 \
    libgtk-3-0 \
    libharfbuzz0b \
    libdbus-1-3 \
    libpango-1.0-0 \
    libcairo2 \
    ca-certificates \
    wget \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd -r browserverless && useradd -r -g browserverless browserverless

RUN wget -q -O /usr/local/bin/browserverless \
    "https://github.com/hieudoanm/browsers/releases/download/latest/browserverless-x86_64-unknown-linux-gnu" \
    && chmod +x /usr/local/bin/browserverless

USER browserverless

ENV RUST_LOG=info
ENV PORT=${PORT}

EXPOSE ${PORT}

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:${PORT}/api/v1/health

ENTRYPOINT ["browserverless"]
CMD browserverless serve --bind 0.0.0.0:${PORT}