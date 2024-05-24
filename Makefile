SHELL = /bin/bash

gocd:
	@ rm -rf ./gocd/generated-pipelines
	@ mkdir -p ./gocd/generated-pipelines
	@ cd ./gocd/templates && jb install && jb update
	@ find . -type f \( -name '*.libsonnet' -o -name '*.jsonnet' \) -print0 | xargs -n 1 -0 jsonnetfmt -i
	@ find . -type f \( -name '*.libsonnet' -o -name '*.jsonnet' \) -print0 | xargs -n 1 -0 jsonnet-lint -J ./gocd/templates/vendor
	@ cd ./gocd/templates && jsonnet --ext-code output-files=true -J vendor -m ../generated-pipelines ./peanutbutter.jsonnet
	@ cd ./gocd/generated-pipelines && find . -type f \( -name '*.yaml' \) -print0 | xargs -n 1 -0 yq -p json -o yaml -i
.PHONY: gocd
