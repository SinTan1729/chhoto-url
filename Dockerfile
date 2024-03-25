FROM scratch
ARG TARGETARCH

COPY .docker/$TARGETARCH/chhoto-url /chhoto-url
COPY ./resources /resources

ENTRYPOINT ["/chhoto-url"]

