{
  description = "loadbench";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    crane,
  }: let
    system = "x86_64-linux";
    pkgs =
      import nixpkgs
      {
        overlays = [rust-overlay.overlays.default];
        system = system;
      };
    rust = pkgs.rust-bin.nightly.latest.default;
    craneLib = crane.lib.${system};
    src = craneLib.cleanCargoSource (craneLib.path ./.);
    commonArgs = {
      inherit src;
    };
    cargoArtifacts = craneLib.buildDepsOnly commonArgs;
  in {
    packages.${system} = {
      loadbench = craneLib.buildPackage (commonArgs
        // {
          inherit cargoArtifacts;
        });

      loadbench-doc = craneLib.cargoDoc (commonArgs
        // {
          inherit cargoArtifacts;
        });
    };

    checks.${system} = {
      inherit (self.packages.${system}) loadbench loadbench-doc;

      # Run clippy (and deny all warnings) on the crate source,
      # again, resuing the dependency artifacts from above.
      #
      # Note that this is done as a separate derivation so that
      # we can block the CI if there are issues here, but not
      # prevent downstream consumers from building our crate by itself.
      loadbench-clippy = craneLib.cargoClippy (commonArgs
        // {
          inherit cargoArtifacts;
          cargoClippyExtraArgs = "--all-targets -- --deny warnings";
        });

      # Check formatting
      loadbench-fmt = craneLib.cargoFmt {
        inherit src;
      };
    };

    formatter.${system} = pkgs.alejandra;

    devShells.${system}.default = pkgs.mkShell {
      packages = [
        (rust.override {
          extensions = ["rust-src" "rustfmt"];
        })
        pkgs.cargo-udeps
        pkgs.cargo-edit
        pkgs.cargo-flamegraph
      ];
    };
  };
}
