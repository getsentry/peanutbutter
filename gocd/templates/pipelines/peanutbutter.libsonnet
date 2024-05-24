// local getsentry = import 'github.com/getsentry/gocd-jsonnet/libs/getsentry.libsonnet';
local gocdtasks = import 'github.com/getsentry/gocd-jsonnet/libs/gocd-tasks.libsonnet';

function(region) {
  environment_variables: {
    SENTRY_REGION: region,
  },
  lock_behavior: 'unlockWhenFinished',
  materials: {
    peanutbutter_repo: {
      git: 'git@github.com:getsentry/peanutbutter.git',
      shallow_clone: true,
      branch: 'master',
      destination: 'peanutbutter',
    },
  },
  stages: [
    {
      checks: {
        fetch_materials: true,
        jobs: {
          checks: {
            timeout: 1200,
            elastic_profile_id: 'peanutbutter',
            tasks: [
              gocdtasks.script(importstr '../bash/cloudbuild-checks.sh'),
            ],
          },
        },
      },
    },
    {
      deploy: {
        approval: {
          type: 'manual',
        },
        fetch_materials: true,
        jobs: {
          deploy: {
            timeout: 1200,
            elastic_profile_id: 'peanutbutter',
            environment_variables: {
              LABEL_SELECTOR: 'service=peanutbutter',
            },
            tasks: [
              gocdtasks.script(importstr '../bash/deploy.sh'),
            ],
          },
        },
      },
    },
  ],
}
