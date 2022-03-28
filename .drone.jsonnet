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
        "cargo build --verbose --all --release",
        "cargo test --verbose --all"
      ]
    }
  ]
};

[
  Pipeline("rust-1-59", "rust:1.59"),
]
