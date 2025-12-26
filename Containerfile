FROM scratch
MAINTAINER Eliseo eliseo.cailloux@wuespace.de

# add Tini for signal handling
ENV TINI_VERSION=v0.19.0
ADD --chmod=0755 https://github.com/krallin/tini/releases/download/${TINI_VERSION}/tini-static /tini

COPY target/x86_64-unknown-linux-musl/release/database /database
ENTRYPOINT ["/tini", "--", "/database"]
