{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    fenix.url = "github:nix-community/fenix";
  };

  outputs = {
    self,
    flake-utils,
    naersk,
    nixpkgs,
    fenix,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = (import nixpkgs) {
          inherit system;
          overlays = [fenix.overlays.default];
        };

        naersk' = pkgs.callPackage naersk {};
      in rec {
        defaultPackage = naersk'.buildPackage {
          src = ./.;
        };

        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            alejandra
            rust-analyzer
            (pkgs.fenix.fromToolchainFile{
              file = ./rust-toolchain.toml;
              sha256 = "sha256-VZZnlyP69+Y3crrLHQyJirqlHrTtGTsyiSnZB8jEvVo=";
            })

            # project specific
            tio # serial monitor utility
            bashmount # QOL utility to mount the pico on boot mode
            flip-link # stack overflow protection
            elf2uf2-rs # used for loading uf2 over usb
            probe-rs-tools # programming, logging and debugging with a debug probe
          ];
        };
      }
    );
}
