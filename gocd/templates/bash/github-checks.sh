#!/bin/bash

/devinfra/scripts/checks/githubactions/checkruns.py \
  getsentry/peanutbutter \
  ${GO_REVISION_PEANUTBUTTER_REPO} \
  'Tests'
