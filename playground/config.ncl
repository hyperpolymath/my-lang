# SPDX-License-Identifier: PMPL-1.0-or-later
# config.ncl - My-Lang Playground Configuration
# Type-safe configuration using Nickel
# See: https://nickel-lang.org

{
  # Project metadata
  project = {
    name = "mylang-playground",
    version = "1.0.0",
    description = "Playground for the My-Lang progressive language family",
    license = "PMPL-1.0-or-later",
    author = "Jonathan D.A. Jewell",
    repository = "https://github.com/hyperpolymath/mylang-playground",
  },

  # Dialect configuration
  dialects = {
    root = "hives",
    names = [
      "me",
      "solo",
      "duet",
      "ensemble",
    ],
    descriptions = {
      me = "Fundamental concepts for beginners",
      solo = "Systems programming foundation",
      duet = "AI-assisted pair programming",
      ensemble = "AI-native multi-agent applications",
    },
  },

  # Documentation configuration
  docs = {
    readme = "README.adoc",
    hives_guide = "hives/README.adoc",
    maintainer_guide = "MAINTAINERS.adoc",
  },

  # Development settings
  development = {
    default_tasks = [
      "build",
      "lint",
      "fmt",
      "test",
    ],
    watch_dirs = [
      "hives",
      "hooks",
    ],
  },

  # Computed values
  full_name = project.name ++ " v" ++ project.version,
  dialect_count = 4,
}
