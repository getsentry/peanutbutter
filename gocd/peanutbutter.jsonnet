local peanutbutter = import './pipelines/peanutbutter.libsonnet';
local pipedream = import 'github.com/getsentry/gocd-jsonnet/libs/pipedream.libsonnet';

local pipedream_config = {
  // Name of your service
  name: 'peanutbutter',

  // The materials you'd like the pipelines to watch for changes
  materials: {
    peanutbutter_repo: {
      git: 'git@github.com:getsentry/peanutbutter.git',
      shallow_clone: true,
      branch: 'master',
      destination: 'peanutbutter',
    },
  },

  // Add rollback
  rollback: {
    material_name: 'peanutbutter_repo',
    stage: 'deploy',
    elastic_profile_id: 'peanutbutter',
    // TODO: Remove the final_stage once we have several deploys with pipeline-complete stage
    final_stage: 'deploy',
  },

  // Set to true to auto-deploy changes (defaults to true)
  auto_deploy: false,
};

// Then call pipedream.render() to generate the set of pipelines for
// a getsentry "pipedream".
pipedream.render(pipedream_config, peanutbutter)
