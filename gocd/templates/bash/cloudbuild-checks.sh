#!/bin/bash

/devinfra/scripts/checks/googlecloud/checkcloudbuild.py \
  ${GO_REVISION_PEANUTBUTTER_REPO} \
  sentryio \
  "us-central1-docker.pkg.dev/sentryio/peanutbutter/image"
