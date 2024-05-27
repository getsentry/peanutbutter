#!/bin/bash

/devinfra/scripts/checks/githubactions/checkruns.py \
  getsentry/peanutbutter \
  ${GO_REVISION_SYMBOLICATOR_REPO} \
  'Tests' \
  'Tests on ubuntu-latest'
