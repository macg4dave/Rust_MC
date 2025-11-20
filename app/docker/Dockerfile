# Multi-stage Dockerfile: build a Linux release binary inside the builder
# stage so the resulting image runs on Linux regardless of host OS.

FROM rust:1 AS builder
WORKDIR /work

# Copy the repository into the build image and build the release binary.
COPY . /work
WORKDIR /work/app

# Ensure minimal packages for building. Keep layers small by cleaning apt lists.
RUN apt-get update \
    && apt-get install -y --no-install-recommends build-essential pkg-config ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Build the release binary inside the builder. This produces a Linux executable
# that will be copied into the final runtime image.
RUN cargo build --release

FROM debian:stable-slim AS runtime
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates attr file tzdata \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /work

# Copy everything from the builder so the runtime image contains the built
# binary at the path the tests expect (/work/app/target/release/fileZoom).
COPY --from=builder /work /work

# If an in-image permission script exists, run it now (apply randomized perms inside image)
RUN if [ -x /work/app/scripts/apply_permissions_in_image.sh ]; then /bin/bash /work/app/scripts/apply_permissions_in_image.sh; fi

WORKDIR /work/app
RUN chmod +x scripts/*.sh || true

# Default command: run the built release binary (cargo produced a workspace-level
# target at `/work/target/release/fileZoom`, so use that path which is present
# regardless of whether the workspace or crate-level build was used).
CMD ["/work/target/release/fileZoom"]
