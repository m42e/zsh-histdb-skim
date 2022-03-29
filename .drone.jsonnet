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
        "mkdir dist",
        "cp target/release/zsh-histdb-skim dist/zsh-histdb-skim-linux-x64",
        "cargo test --verbose --all"
      ]
    },
    {
      name: "release",
      image: "alpine:latest",
      pull: "if-not-exists",
      environment:{
        GH_TOKEN: {
          from_secret: "github_release",
        },
      },
      commands: [
        "echo $GH_TOKEN",
        "export GH_REPO=m42e/zsh-histdb-skim",
        "apk --no-cache add wget tar",
        "wget https://github.com/cli/cli/releases/download/v2.6.0/gh_2.6.0_linux_amd64.tar.gz",
        "tar -zxvf gh_2.6.0_linux_amd64.tar.gz",
        "chmod a+x gh_2.6.0_linux_amd64/bin/gh",
        "gh_2.6.0_linux_amd64/bin/gh release create ${DRONE_TAG} dist/*"
      ],
      settings: {
        DRONE_REPO_OWNER: "m42e",
        DRONE_REPO_NAME: "zsh-histdb-skim",
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
