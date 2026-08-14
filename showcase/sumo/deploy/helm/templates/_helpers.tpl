{{- define "sumo.image" -}}
{{- $root := index . 0 -}}
{{- $image := index . 1 -}}
{{- $lockedDigest := get $root.Values.global.imageDigests $image.repository | default "" -}}
{{- $digest := $image.digest | default $lockedDigest -}}
{{- $tag := default $image.tag $root.Values.global.veoveoTag -}}
{{- include "veoveo-extension.image" (dict
    "registry" $root.Values.global.veoveoRegistry
    "production" $root.Values.global.production
    "image" (dict "repository" $image.repository "tag" $tag "digest" $digest)
  ) -}}
{{- end }}
