# SPDX-License-Identifier: MPL-2.0
# Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
#
# ClusterFuzzLite / OSS-Fuzz build image. The ClusterFuzzLite actions
# (google/clusterfuzzlite) build the project from `.clusterfuzzlite/Dockerfile`
# by hard-coded convention, so this file MUST be named `Dockerfile`.
FROM gcr.io/oss-fuzz-base/base-builder-rust
RUN apt-get update && apt-get install -y make autoconf automake libtool
COPY . $SRC/my-lang
WORKDIR $SRC/my-lang
COPY .clusterfuzzlite/build.sh $SRC/
