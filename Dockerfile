FROM scratch
ARG ARCH=linux/amd64

COPY $ARCH/chhoto-url /chhoto-url
COPY ./resources /resources

ENTRYPOINT ["/chhoto-url"]

