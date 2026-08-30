FROM rust:1-bookworm

ARG CARGO_LLVM_COV_VERSION=0.6.19

RUN apt-get update \
    && apt-get install -y --no-install-recommends git \
    && rm -rf /var/lib/apt/lists/* \
    && rustup component add llvm-tools-preview \
    && cargo install cargo-llvm-cov --locked --version "${CARGO_LLVM_COV_VERSION}"

WORKDIR /workspace

CMD ["sleep", "infinity"]
