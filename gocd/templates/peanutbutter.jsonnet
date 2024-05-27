local peanutbutter = import './pipelines/peanutbutter.libsonnet';
local pipedream = import 'github.com/getsentry/gocd-jsonnet/libs/pipedream.libsonnet';

local pipedream_config = {
  // Name of your service
  name: 'peanutbutter',
  // Peanutbutter should only run on `s4s`, `us`, and `de`.
  exclude_regions: ['customer-1', 'customer-2', 'customer-3', 'customer-4', 'customer-5', 'customer-6', 'customer-7'],

  // The materials you'd like the pipelines to watch for changes
  materials: {
    peanutbutter_repo: {
      git: 'git@github.com:getsentry/peanutbutter.git',
      shallow_clone: true,
      branch: 'master',
      destination: 'peanutbutter',
    },
  },
};

// Then call pipedream.render() to generate the set of pipelines for
// a getsentry "pipedream".
pipedream.render(pipedream_config, peanutbutter)
