{
  description = "rust flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default";
    flake-parts.url = "github:hercules-ci/flake-parts";
    fenix = {
      url = "github:nix-community/fenix/monthly";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    naersk = {
      url = "github:nix-community/naersk";
      inputs.fenix.follows = "fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } ({ ... }: {
      systems = import inputs.systems;
      perSystem = { pkgs, system, inputs', ... }: 
      let
        fpkgs = inputs'.fenix.packages;
        rust-toolchain = fpkgs.stable.toolchain;
      in {
        devShells.default =
        pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            rust-toolchain
            rust-analyzer

            # extra cargo tools
            cargo-edit
            cargo-expand
            cargo-show-asm
            cargo-binutils
          ];

          # set the rust src for rust_analyzer
          RUST_SRC_PATH = "${rust-toolchain}/lib/rustlib/src/rust/library";
        };
        packages.default = 
        let
          cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        in
        (inputs.naersk.lib.${system}.override {
          cargo = rust-toolchain;
          rustc = rust-toolchain;
        }).buildPackage {
          src = ./.;
          # set mainProgram variable for lib.getExe to work
          overrideMain = old: {
            meta = (old.meta or {}) // { mainProgram = cargoToml.package.name; };
          };
        };
      };
    });
}

