# GNURUST.GNUCOBOL-TESTSUITE.DIAGNOSTIC-UNBLOCKED.1 court image.
#
# Built ONLY inside the project-scoped isolated daemon from the locally-imported minimal Ubuntu
# base (noble server cloud image, hash-pinned) — no registry pulls. The oracle is built from the
# ADMITTED pinned source (gnucobol-3.2.tar.lz, sha256
# 8ecc77d0a4c9401618b8b99adf2050adef14767916767c54bb42341f0ab504fb). Heavy artifacts live on bind
# mounts under /work (the host project dir), NOT in image layers. autoconf provides autom4te, the
# upstream mechanism that regenerates the Autotest `testsuite`.
#
# This lane NEVER touches the pristine suite: it runs the SAME tree with a mechanically patched
# tests/testsuite.src and the UPSTREAM `make -C tests testsuite` regeneration.
ARG BASE_IMAGE
FROM ${BASE_IMAGE}

ENV LC_ALL=C.UTF-8 \
    LANG=C.UTF-8 \
    TZ=UTC0 \
    SOURCE_DATE_EPOCH=725846400 \
    DEBIAN_FRONTEND=noninteractive

RUN apt-get update -qq \
 && apt-get install -y -qq --no-install-recommends \
      build-essential \
      make \
      gawk \
      perl \
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
      autoconf \
      automake \
      libtool \
      texinfo \
      patch \
 && rm -rf /var/lib/apt/lists/*

RUN useradd -m -s /bin/bash bench || true

COPY run.sh /usr/local/bin/gnucobol-testsuite-diag-unblocked-run.sh
RUN chmod +x /usr/local/bin/gnucobol-testsuite-diag-unblocked-run.sh

WORKDIR /work
