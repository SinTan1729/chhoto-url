#!/bin/env bash

if [ "$1" == "dev" ]; then
    name="chhoto-url"
    docker build -t $name .
    docker tag $name sintan1729/$name:dev
    docker push sintan1729/$name:dev

elif [ "$1" == "release" ]; then
    v_patch=$(cat actix/Cargo.toml | sed -rn 's/^version = "(.+)"$/\1/p')
    v_minor=$(echo $v_patch | sed -rn 's/^(.+\..+)\..+$/\1/p')
    v_major=$(echo $v_minor | sed -rn 's/^(.+)\..+$/\1/p')

    name="chhoto-url"

    docker build -t $name .

    for tag in $v_major $v_minor $v_patch latest
    do
        docker tag $name sintan1729/$name:$tag
    done

    echo "Do you want to push these to Docker Hub?"
    select yn in "Yes" "No";
    do
        if [ "$yn"="Yes" ]; then
            for tag in $v_major $v_minor $v_patch latest
            do
                docker push sintan1729/$name:$tag
            done
        else
            echo "Okay! Not pushing."
        fi
        break
    done
fi

