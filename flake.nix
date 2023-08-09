{
  description = "loadbench";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
  }: let
    system = "x86_64-linux";
    pkgs =
      import nixpkgs
      {
        overlays = [rust-overlay.overlays.default];
        system = system;
      };
    rust = pkgs.rust-bin.nightly.latest.default;
  in {
    formatter.${system} = pkgs.alejandra;

    devShells.${system}.default = pkgs.mkShell {
      packages = [
        (rust.override {
          extensions = ["rust-src" "rustfmt"];
        })
        pkgs.cargo-udeps
        pkgs.cargo-edit
      ];
    };
  };
}
