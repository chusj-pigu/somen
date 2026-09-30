ARG JBOX_BASE_IMAGE
FROM ${JBOX_BASE_IMAGE}

# The Jbox Debian runtime has no Rust toolchain or native build tools.
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        cmake \
        jq \
        pkg-config \
        ripgrep \
        unzip \
    && rm -rf /var/lib/apt/lists/*

# Keep the toolchain readable by non-root guest users. Their Cargo download
# caches remain in their own home directories instead of the root-owned image.
ARG RUST_VERSION=1.98.1
ENV RUSTUP_HOME=/opt/rustup
ENV PATH="/opt/cargo/bin:${PATH}"
RUN curl --proto '=https' --tlsv1.2 -fsSL https://sh.rustup.rs -o /tmp/rustup-init.sh \
    && CARGO_HOME=/opt/cargo sh /tmp/rustup-init.sh -y --profile minimal \
        --default-toolchain "${RUST_VERSION}" --no-modify-path \
    && CARGO_HOME=/opt/cargo rustup component add clippy rustfmt \
    && rm -f /tmp/rustup-init.sh

# Debian 12 provides flatc 2.0.8, but somen-pod5 uses FlatBuffers 25.12.19.
# Pin and verify the official Linux compiler release matching that dependency.
RUN test "$(dpkg --print-architecture)" = amd64 \
    && curl --proto '=https' --tlsv1.2 -fsSL --retry 3 \
        'https://github.com/google/flatbuffers/releases/download/v25.12.19/Linux.flatc.binary.clang%2B%2B-18.zip' \
        -o /tmp/flatc.zip \
    && echo '50c1915deeeb714f2a05c8ec795bd1af898d251a62e2774067703b29188efc90  /tmp/flatc.zip' | sha256sum -c - \
    && unzip -p /tmp/flatc.zip flatc > /usr/local/bin/flatc \
    && chmod 0755 /usr/local/bin/flatc \
    && flatc --version | grep -Fx 'flatc version 25.12.19' \
    && rm -f /tmp/flatc.zip
