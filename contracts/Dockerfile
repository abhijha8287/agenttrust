# Solana/Anchor build environment for the agent_trust program.
#
# Windows-native Solana tooling is unreliable (the BPF toolchain is Linux-
# first), and this environment has no WSL general-purpose distro installed.
# Building inside a container sidesteps that entirely — nothing here touches
# the host beyond Docker.
#
# Pinned versions (not "latest"): Anchor 1.0.0 shipped breaking changes in
# April 2026 (@anchor-lang/core replacing @coral-xyz/anchor, Surfpool
# replacing solana-test-validator). Anchor 0.30.1 + Solana CLI 1.18.17 is the
# well-documented, stable combination — boring by default, not bleeding edge.

FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive
ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo
ENV PATH=/usr/local/cargo/bin:/root/.local/share/solana/install/active_release/bin:/root/.avm/bin:$PATH

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    libudev-dev \
    curl \
    ca-certificates \
    git \
    && rm -rf /var/lib/apt/lists/*

# Rust (Solana 1.18.17 needs a reasonably recent stable toolchain)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
RUN rustup component add rustfmt clippy

# Solana CLI 1.18.17 (the CLI is now the Agave release, maintained by Anza —
# release.solana.com was retired in favor of release.anza.xyz)
RUN curl -sSfL https://release.anza.xyz/v1.18.17/install -o /tmp/solana-install.sh \
    && sh /tmp/solana-install.sh \
    && rm /tmp/solana-install.sh

# Node.js 20 LTS + Yarn (Anchor's TS test framework)
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs \
    && npm install -g yarn

# Anchor CLI 0.30.1 via AVM
RUN cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
RUN avm install 0.30.1 && avm use 0.30.1

# anchor build's IDL generation hardcodes `cargo +nightly`. As of
# nightly-2025-04-16, rustc removed the unstable proc_macro::SourceFile API
# that anchor-syn 0.30.1 (and proc-macro2 <1.0.95) depend on — fixed properly
# in Anchor 0.31.1, but that requires Solana CLI 2.1.0, a much bigger jump
# than justified just to fix IDL generation. Instead: install a nightly from
# before the breaking change and re-link plain "nightly" to point at it, so
# anchor's hardcoded `+nightly` picks up a compatible compiler without
# touching anything else in this already-working toolchain.
# `rustup toolchain link` refuses "nightly" as a reserved channel name, so
# this uses a raw filesystem symlink instead — rustup only validates names
# in the `link` subcommand itself, not when it later scans the toolchains
# directory, so `cargo +nightly` resolves the symlink same as any other
# installed toolchain.
RUN rustup toolchain install nightly-2024-08-01 \
    && rm -rf "$RUSTUP_HOME/toolchains/nightly-x86_64-unknown-linux-gnu" \
    && ln -s "$RUSTUP_HOME/toolchains/nightly-2024-08-01-x86_64-unknown-linux-gnu" \
             "$RUSTUP_HOME/toolchains/nightly-x86_64-unknown-linux-gnu"

WORKDIR /workspace

CMD ["bash"]
