#!/bin/bash
set -e

docker build \
	--target builder \
	--tag jsalverda/jarvis-alpha-innotec-exporter:dlc-builder \
	--cache-from jsalverda/jarvis-alpha-innotec-exporter:dlc-builder \
	--build-arg BUILDKIT_INLINE_CACHE=1 .
docker push jsalverda/jarvis-alpha-innotec-exporter:dlc-builder

docker build \
	--target runtime \
	--tag jsalverda/jarvis-alpha-innotec-exporter:dlc \
	--cache-from jsalverda/jarvis-alpha-innotec-exporter:dlc-builder \
	--cache-from jsalverda/jarvis-alpha-innotec-exporter:dlc \
	--build-arg BUILDKIT_INLINE_CACHE=1 .
docker push jsalverda/jarvis-alpha-innotec-exporter:dlc
