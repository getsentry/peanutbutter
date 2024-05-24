#!/bin/bash

eval $(/devinfra/scripts/regions/project_env_vars.py --region="${SENTRY_REGION}")
/devinfra/scripts/k8s/k8stunnel
/devinfra/scripts/k8s/k8s-deploy.py \
  --type="statefulset" \
  --label-selector="${LABEL_SELECTOR}" \
  --image="us-central1-docker.pkg.dev/sentryio/peanutbutter/image:${GO_REVISION_PEANUTBUTTER_REPO}" \
  --container-name="peanutbutter" \
  --container-name="cleanup"
