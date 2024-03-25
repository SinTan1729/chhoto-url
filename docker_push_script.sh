#!/bin/env bash

if [ "$1" == "dev" ]; then
    docker buildx build --push --tag sintan1729/$name:dev .

elif [ "$1" == "release" ]; then
    v_patch=$(cat actix/Cargo.toml | sed -rn 's/^version = "(.+)"$/\1/p')
    v_minor=$(echo $v_patch | sed -rn 's/^(.+\..+)\..+$/\1/p')
    v_major=$(echo $v_minor | sed -rn 's/^(.+)\..+$/\1/p')
    
    make build
    name="chhoto-url"
    docker buildx build --push --tag sintan1729/$name:$v_major --tag sintan1729/$v_minor: \
        --tag sintan1729/$name:$v_patch --tag sintan1729/$name:latest --platform linux/amd64,linux/arm64,linux/arm/v7 -f Dockerfile.multiarch .
fi

