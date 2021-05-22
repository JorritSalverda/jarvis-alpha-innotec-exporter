#!/bin/bash
set -e

docker build \
	--target base \
	--tag jsalverda/jarvis-alpha-innotec-exporter:dlc-base \
	--cache-from jsalverda/jarvis-alpha-innotec-exporter:dlc-base \
	--build-arg BUILDKIT_INLINE_CACHE=1 .
docker push jsalverda/jarvis-alpha-innotec-exporter:dlc-base

docker build \
	--target planner \
	--tag jsalverda/jarvis-alpha-innotec-exporter:dlc-planner \
  --cache-from jsalverda/jarvis-alpha-innotec-exporter:dlc-base \
	--cache-from jsalverda/jarvis-alpha-innotec-exporter:dlc-planner \
	--build-arg BUILDKIT_INLINE_CACHE=1 .
docker push jsalverda/jarvis-alpha-innotec-exporter:dlc-planner

docker build \
	--target cacher \
	--tag jsalverda/jarvis-alpha-innotec-exporter:dlc-cacher \
  --cache-from jsalverda/jarvis-alpha-innotec-exporter:dlc-base \
  --cache-from jsalverda/jarvis-alpha-innotec-exporter:dlc-planner \
	--cache-from jsalverda/jarvis-alpha-innotec-exporter:dlc-cacher \
	--build-arg BUILDKIT_INLINE_CACHE=1 .
docker push jsalverda/jarvis-alpha-innotec-exporter:dlc-cacher

# docker build \
# 	--target builder \
# 	--tag jsalverda/jarvis-alpha-innotec-exporter:dlc-builder \
#   --cache-from jsalverda/jarvis-alpha-innotec-exporter:dlc-base \
#   --cache-from jsalverda/jarvis-alpha-innotec-exporter:dlc-planner \
# 	--cache-from jsalverda/jarvis-alpha-innotec-exporter:dlc-cacher \
# 	--cache-from jsalverda/jarvis-alpha-innotec-exporter:dlc-builder \
# 	--build-arg BUILDKIT_INLINE_CACHE=1 .
# docker push jsalverda/jarvis-alpha-innotec-exporter:dlc-builder

docker build \
	--tag jsalverda/jarvis-alpha-innotec-exporter:dlc \
  --cache-from jsalverda/jarvis-alpha-innotec-exporter:dlc-base \
  --cache-from jsalverda/jarvis-alpha-innotec-exporter:dlc-planner \
	--cache-from jsalverda/jarvis-alpha-innotec-exporter:dlc-cacher \
	--cache-from jsalverda/jarvis-alpha-innotec-exporter:dlc \
	--build-arg BUILDKIT_INLINE_CACHE=1 .
docker push jsalverda/jarvis-alpha-innotec-exporter:dlc
