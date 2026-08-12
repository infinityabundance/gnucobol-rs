# GNURUST.VALID-PROGRAMS / GNURUST.CORPUS court image — GnuCOBOL 3.2 oracle + Rust toolchain +
# the corpus CLI (gnucobol-rs-corpus) + the bench CLI (gnucobol-rs-bench).
#
# Built ONLY inside the project-scoped isolated daemon from the locally-imported minimal Ubuntu
# base (noble server cloud image, hash-pinned). No registry pulls for the base image. The oracle
# is built from the ADMITTED pinned source (gnucobol-3.2.tar.lz, sha256
# 8ecc77d0a4c9401618b8b99adf2050adef14767916767c54bb42341f0ab504fb) — never a distribution
# package. Heavy artifacts (oracle prefix, cargo/rustup homes, target dir) live on bind mounts
# under /work (the host project dir), NOT in image layers.
#
# Build: docker build --build-arg BASE_IMAGE=gnucobol-rs-valid-corpus/ubuntu-base:<hash> -t <tag> .

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

# The harness entry points are bind-mounted at run time (kept out of the image so the image is
# reusable across harness versions without rebuilds).
COPY run.sh /usr/local/bin/valid-corpus-run.sh
COPY held-out-run.sh /usr/local/bin/held-out-run.sh
RUN chmod +x /usr/local/bin/valid-corpus-run.sh /usr/local/bin/held-out-run.sh

WORKDIR /work
