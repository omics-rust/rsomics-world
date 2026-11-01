FROM docker.io/library/ubuntu@sha256:78d4744ead1f5b19b04d0df558ab182a5300903900755e515b2ca4d10eb2044d
ARG RSOMICS_SIGNAL_RECIPE_SHA256
LABEL org.rsomics.signal.base-manifest="sha256:78d4744ead1f5b19b04d0df558ab182a5300903900755e515b2ca4d10eb2044d"
LABEL org.rsomics.signal.recipe-sha256="${RSOMICS_SIGNAL_RECIPE_SHA256}"
RUN set -eu; \
    mkdir -p /usr/local/share/rsomics-system-closure/archives; \
    apt-get update; \
    apt-get --download-only --no-install-recommends -y install python3 git binutils libstdc++6 libgomp1; \
    cp /var/cache/apt/archives/*.deb /usr/local/share/rsomics-system-closure/archives/; \
    apt-get --no-download --no-install-recommends -y install python3 git binutils libstdc++6 libgomp1; \
    dpkg-query -W -f='${Package}\t${Version}\t${Architecture}\n' > /usr/local/share/rsomics-system-closure/packages.tsv; \
    cp -a /etc/apt /usr/local/share/rsomics-system-closure/apt-config; \
    cp -a /var/lib/apt/lists /usr/local/share/rsomics-system-closure/apt-lists; \
    cd /usr/local/share/rsomics-system-closure; \
    find archives apt-config apt-lists -type d -exec chmod 755 '{}' +; \
    find archives apt-config apt-lists -type f -exec chmod 644 '{}' +; \
    find archives apt-config apt-lists -type f -exec sha256sum '{}' + > bodies.sha256
