# Image for running fileZoom tests with fixtures baked in
# Builds a container that contains the repository at /work and fixtures at /work/app/tests/fixtures
FROM debian:stable-slim

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates attr file tzdata \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /work

# Copy repo into image. This preserves the fixtures under app/tests/fixtures
COPY . /work

# If an in-image permission script exists, run it now (apply randomized perms inside image)
RUN if [ -x /work/app/scripts/apply_permissions_in_image.sh ]; then /bin/bash /work/app/scripts/apply_permissions_in_image.sh; fi

WORKDIR /work/app

# Make scripts executable
RUN chmod +x scripts/*.sh || true

# The release binary is expected to already exist in the build context (target/release/fileZoom).
# The build helper (run_with_fakefs.sh) can build the binary locally before creating the image
# so the image does not need to compile Rust at build time.

# Default command: run the built binary if present, otherwise fall back to a shell.
CMD ["/work/app/target/release/fileZoom"]
