#!/usr/bin/env bash

INPUT_ARGS=build \
INPUT_REGEX=apple \
INPUT_REPO=mdbook \
INPUT_OWNER=rust-lang \
INPUT_TOKEN=$GITHUB_API_KEY \
node index.js
