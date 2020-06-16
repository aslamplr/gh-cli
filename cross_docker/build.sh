#!/bin/bash
cd cross_docker
docker build -t my/cross:armv7-unknown-linux-gnueabihf -f cross.Dockerfile .
cd ..

