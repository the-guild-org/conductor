variable "DOCKER_REGISTRY" {
  default = ""
}

variable "COMMIT_SHA" {
  default = "local"
}

variable "PUBLISH_VERSION" {
  # Can be "" or the actual version to publish
  default = ""
}

variable "PUBLISH_LATEST" {
  # Can be "" or "1"
  default = ""
}

function "maybe_latest_image_tag" {
  params = [name]
  result = equal("1", PUBLISH_LATEST) ? "${DOCKER_REGISTRY}${name}:latest" : ""
}

function "maybe_version_tag" {
  params = [name]
  result = notequal("", PUBLISH_VERSION) ? "${DOCKER_REGISTRY}${name}:${PUBLISH_VERSION}" : ""
}

function "commit_id_tag" {
  params = [name, tag]
  result = notequal("", tag) ? "${DOCKER_REGISTRY}${name}:${tag}" : ""
}

target "conductor" {
  context = "./"
  dockerfile = "./crates/conductor/docker/Dockerfile"
  tags = [
    commit_id_tag("conductor", COMMIT_SHA),
    maybe_latest_image_tag("conductor"),
    maybe_version_tag("conductor"),
  ]
  labels = {
    "org.opencontainers.image.source" = "https://github.com/the-guild-org/conductor-t2",
    "org.opencontainers.image.authors": "The Guild <contact@the-guild.dev>",
    "org.opencontainers.image.vendor": "The Guild",
    "org.opencontainers.image.url": "https://the-guild.dev/graphql/gateway",
    "org.opencontainers.image.docs": "https://the-guild.dev/graphql/gateway",
    "org.opencontainers.image.version": PUBLISH_VERSION,
    "org.opencontainers.image.revision": COMMIT_SHA,
    "org.opencontainers.image.licenses": "MIT",
    "org.opencontainers.image.title": "Conductor",
    "org.opencontainers.image.description": "Conductor is a robust GraphQL Gateway."
  }
}

group "build" {
  targets = [
    "conductor"
  ]
}
