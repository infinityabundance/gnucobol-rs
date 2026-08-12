# GNURUST.PERFORMANCE court image — GnuCOBOL 3.2 oracle + Rust toolchain + gnucobol-rs-bench.
#
# Built ONLY inside the project-scoped isolated daemon from the locally-imported minimal Ubuntu
# base (noble server cloud image, hash-pinned). The oracle is built from the ADMITTED pinned
# source (gnucobol-3.2.tar.lz, sha256
# 8ecc77d0a4c9401618b8b99adf2050adef14767916767c54bb42341f0ab504fb) — never a distribution
# package. Heavy artifacts (oracle prefix, cargo/rustup homes, target dir) live on bind mounts
# under /work (the host project dir), NOT in image layers.
#
# Build: docker build --build-arg BASE_IMAGE=gnucobol-rs-performance/ubuntu-base:<hash> -t <tag> .

ARG BASE_IMAGE
FROM ${BASE_IMAGE}

ENV LC_ALL=C.UTF-8 \
    LANG=C.UTF-8 \
    TZ=UTC0 \
    SOURCE_DATE_EPOCH=725846400 \
    DEBIAN_FRONTEND=noninteractive

# Native build/runtime dependencies, installed EXPLICITLY (no distro gnucobol anywhere).
# APT_PACKAGES=0 is used on machines whose rootless daemon cannot reach a package mirror
# (no tap devices / no NAT): there the court image is based on the package-complete
# ccvs85 court image and this step is skipped.
ARG APT_PACKAGES=1
RUN if [ "$APT_PACKAGES" != "0" ]; then apt-get update -qq \
 && apt-get install -y -qq --no-install-recommends \
      build-essential \
      make \
      libgmp-dev \
      libdb5.3-dev \
      libncurses-dev \
      libxml2-dev \
      libjson-c-dev \
      libicu-dev \
      zlib1g-dev \
      libltdl-dev \
      bison \
      flex \
      lzip \
      curl \
      ca-certificates \
      file \
      coreutils \
      gzip \
      xz-utils \
      procps \
      time \
      python3 \
 && rm -rf /var/lib/apt/lists/*; fi

RUN useradd -m -s /bin/bash bench || true

COPY run.sh /usr/local/bin/performance-run.sh
RUN chmod +x /usr/local/bin/performance-run.sh

WORKDIR /work
