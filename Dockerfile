ARG BUILDPLATFORM

FROM --platform=$BUILDPLATFORM tonistiigi/xx AS xx

FROM --platform=$BUILDPLATFORM rust:alpine AS chef
COPY --from=xx / /

RUN apk add clang lld
RUN cargo install cargo-chef 
WORKDIR /app

FROM chef AS depcacher
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo fetch

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

# Setup the environment for the target platform
ARG TARGETPLATFORM
RUN xx-cargo --setup-target-triple

# Build dependencies
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    xx-cargo chef cook --release --recipe-path recipe.json

# Build the application
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    xx-cargo build --release

# Verify the build
RUN xx-verify --static target/$(xx-cargo --print-target-triple)/release/trashbot

# Copy the executable to an easily-findable path
RUN mkdir -p /app/target/release
RUN cp target/$(xx-cargo --print-target-triple)/release/trashbot /app/target/release

FROM scratch AS runtime
COPY --from=builder /app/target/release/trashbot /trashbot
ENTRYPOINT ["/trashbot"]
