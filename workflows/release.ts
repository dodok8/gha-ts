// Cross-platform build, GitHub Release assets, and npm publishing
// Triggered by tags created by release-plz
import { getAction, Job, Workflow } from "../generated/index.js";

const checkout = getAction("actions/checkout@v5");
const rustToolchain = getAction("dtolnay/rust-toolchain@stable");
const uploadArtifact = getAction("actions/upload-artifact@v4");
const downloadArtifact = getAction("actions/download-artifact@v4");
const ghRelease = getAction("softprops/action-gh-release@v2");
const setupNode = getAction("actions/setup-node@v4");

// --- Job 1: Cross-platform build ---
const build = new Job("${{ matrix.target.runner }}", {
  strategy: {
    "fail-fast": false,
    matrix: {
      target: [
        {
          runner: "ubuntu-latest",
          rust_target: "x86_64-unknown-linux-gnu",
          platform: "linux-x64",
          binary: "gaji",
        },
        {
          runner: "ubuntu-latest",
          rust_target: "aarch64-unknown-linux-gnu",
          platform: "linux-arm64",
          binary: "gaji",
        },
        {
          runner: "macos-latest",
          rust_target: "x86_64-apple-darwin",
          platform: "darwin-x64",
          binary: "gaji",
        },
        {
          runner: "macos-latest",
          rust_target: "aarch64-apple-darwin",
          platform: "darwin-arm64",
          binary: "gaji",
        },
        {
          runner: "windows-latest",
          rust_target: "x86_64-pc-windows-msvc",
          platform: "win32-x64",
          binary: "gaji.exe",
        },
      ],
    },
  },
})
  .addStep(checkout({}))
  .addStep(
    rustToolchain({
      with: {
        targets: "${{ matrix.target.rust_target }}",
      },
    })
  )
  .addStep({
    name: "Install cross-compilation tools",
    if: "matrix.target.rust_target == 'aarch64-unknown-linux-gnu'",
    run: [
      "sudo apt-get update",
      "sudo apt-get install -y gcc-aarch64-linux-gnu",
    ].join("\n"),
  })
  .addStep({
    name: "Build binary",
    run: "cargo build --release --target ${{ matrix.target.rust_target }}",
    env: {
      CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER: "aarch64-linux-gnu-gcc",
    },
  })
  .addStep({
    name: "Create tarball (unix)",
    if: "runner.os != 'Windows'",
    run: [
      "cd target/${{ matrix.target.rust_target }}/release",
      "tar czf gaji-${{ matrix.target.platform }}.tar.gz ${{ matrix.target.binary }}",
    ].join("\n"),
  })
  .addStep({
    name: "Create zip (windows)",
    if: "runner.os == 'Windows'",
    run: "Compress-Archive -Path target/${{ matrix.target.rust_target }}/release/${{ matrix.target.binary }} -DestinationPath target/${{ matrix.target.rust_target }}/release/gaji-${{ matrix.target.platform }}.zip",
    shell: "pwsh",
  })
  .addStep(
    uploadArtifact({
      with: {
        name: "binary-${{ matrix.target.platform }}",
        path: "target/${{ matrix.target.rust_target }}/release/gaji-${{ matrix.target.platform }}.*",
      },
    })
  )
  .addStep({
    name: "Prepare npm platform package (unix)",
    if: "runner.os != 'Windows'",
    run: [
      "mkdir -p npm/platform-${{ matrix.target.platform }}/bin",
      "cp target/${{ matrix.target.rust_target }}/release/${{ matrix.target.binary }} npm/platform-${{ matrix.target.platform }}/bin/",
    ].join("\n"),
  })
  .addStep({
    name: "Prepare npm platform package (windows)",
    if: "runner.os == 'Windows'",
    run: [
      "New-Item -ItemType Directory -Force -Path npm/platform-${{ matrix.target.platform }}/bin",
      "Copy-Item target/${{ matrix.target.rust_target }}/release/${{ matrix.target.binary }} npm/platform-${{ matrix.target.platform }}/bin/",
    ].join("\n"),
    shell: "pwsh",
  })
  .addStep(
    uploadArtifact({
      with: {
        name: "npm-${{ matrix.target.platform }}",
        path: "npm/platform-${{ matrix.target.platform }}/",
      },
    })
  );

// --- Job 2: Upload binaries to GitHub Release ---
const uploadReleaseAssets = new Job("ubuntu-latest", {
  needs: ["build"],
  permissions: {
    contents: "write",
  },
})
  .addStep(
    downloadArtifact({
      with: {
        pattern: "binary-*",
        path: "artifacts",
        "merge-multiple": true,
      },
    })
  )
  .addStep({
    name: "Generate checksums",
    run: ["cd artifacts", "sha256sum * > checksums.txt"].join("\n"),
  })
  .addStep(
    ghRelease({
      with: {
        files: "artifacts/*",
      },
      env: {
        GITHUB_TOKEN: "${{ secrets.GITHUB_TOKEN }}",
      },
    })
  );

// --- Job 3: Publish to npm (Trusted Publishing) ---
const publishNpm = new Job("ubuntu-latest", {
  needs: ["build"],
  permissions: {
    "id-token": "write",
  },
})
  .addStep(checkout({}))
  .addStep(
    setupNode({
      with: {
        "node-version": "22",
        "registry-url": "https://registry.npmjs.org",
      },
    })
  )
  .addStep({
    name: "Upgrade npm",
    run: "npm install -g npm@latest",
  })
  .addStep({
    name: "Sync versions",
    run: "bash scripts/sync-versions.sh",
  })
  .addStep(
    downloadArtifact({
      with: {
        pattern: "npm-*",
        path: "npm-artifacts",
      },
    })
  )
  .addStep({
    name: "Prepare platform packages",
    run: [
      'for dir in npm-artifacts/npm-*/; do',
      '  PLATFORM=$(basename "$dir" | sed "s/npm-//")',
      '  cp -r "$dir"/* "npm/platform-$PLATFORM/"',
      "done",
    ].join("\n"),
  })
  .addStep({
    name: "Publish platform packages",
    run: [
      "for dir in npm/platform-*/; do",
      '  echo "Publishing $(basename $dir)..."',
      '  cd "$dir"',
      "  npm publish --provenance --access public",
      "  cd ../..",
      "done",
    ].join("\n"),
  })
  .addStep({
    name: "Publish main package",
    run: [
      "cd npm/gaji",
      "npm publish --provenance --access public",
    ].join("\n"),
  });

// --- Assemble workflow ---
const workflow = new Workflow({
  name: "Release",
  on: {
    push: {
      tags: ["v*"],
    },
  },
  permissions: {
    contents: "read",
  },
})
  .addJob("build", build)
  .addJob("upload-release-assets", uploadReleaseAssets)
  .addJob("publish-npm", publishNpm);

workflow.build("release");
