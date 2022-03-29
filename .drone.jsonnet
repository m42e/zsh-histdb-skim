local Pipeline(name, image) = {
  kind: "pipeline",
  type: "docker",
  name: name,
  steps: [
    {
      name: "test",
      image: image,
      pull: "if-not-exists",
      commands: [
        "cargo build --verbose --all --release --out-dir dist",
        "cargo test --verbose --all"
      ]
    },
    {
      name: "release",
      image: "plugins/github-release",
      pull: "if-not-exists",
      settings: {
        api_key: {
          from_secret: "github_release",
        },
        files: 'dist/*',
        draft: true,
      },
      when: {
        event: 'tag'
      },
    }
  ]
};

[
  Pipeline("rust-1-59", "rust:1.59"),
]
