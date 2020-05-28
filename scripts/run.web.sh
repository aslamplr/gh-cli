#!/bin/bash
# Run the docker container; run the build.web.sh if you haven't already!

IS_DETACHED=false;
if [[ "$*" =~ --detached ]]; then 
  IS_DETACHED=true;
fi

CLEANUP=--rm;
if [[ "$*" =~ --no-rm ]]; then 
  CLEANUP=;
fi

EXTERNAL_PORT=3030;
if [[ "$*" =~ --port[[:space:]|=]([0-9]{2,5}) ]]; then
  EXTERNAL_PORT=${BASH_REMATCH[1]};
fi

docker run ${CLEANUP} -d=${IS_DETACHED} -t -i -p ${EXTERNAL_PORT}:3030 -e PORT=3030 gh-web:latest
