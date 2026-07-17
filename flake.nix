{
  description = "SCAL";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
        runtime = pkgs.rustPlatform.buildRustPackage {
          pname = "scal-runtime";
          version = "1.0.1";

          src = ./.;

          cargoLock.lockFile = ./Cargo.lock;

          cargoBuildFlags = [
            "-p"
            "scal-runtime"
          ];

          nativeBuildInputs = with pkgs; [
            pkg-config
            clang
          ];

          buildInputs = with pkgs; [
            ffmpeg

            wayland
            libxkbcommon

            xorg.libX11
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr
            xorg.libxcb

            alsa-lib
            libpulseaudio
          ];
        };
      in
      {
        packages = {
          default = runtime;
          runtime = runtime;
        };

        apps.default = {
          type = "app";
          program = "${runtime}/bin/scal-runtime";
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            rust-analyzer
            rustfmt
            clippy
            cargo
            rustc

            pkg-config
            ffmpeg

            wayland
            libxkbcommon
            xorg.libX11
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr
            xorg.libxcb

            clang
            llvmPackages.libclang

            alsa-lib
            alsa-lib.dev
            libpulseaudio
          ];

          shellHook = ''
            export LIBCLANG_PATH="${pkgs.llvmPackages.libclang.lib}/lib"

            export LD_LIBRARY_PATH="${
              pkgs.lib.makeLibraryPath [
                pkgs.wayland
                pkgs.libxkbcommon
                pkgs.xorg.libX11
                pkgs.xorg.libXcursor
                pkgs.xorg.libXi
                pkgs.xorg.libXrandr
                pkgs.xorg.libxcb
                pkgs.alsa-lib
                pkgs.libpulseaudio
              ]
            }:$LD_LIBRARY_PATH"

            export PKG_CONFIG_PATH="${pkgs.alsa-lib.dev}/lib/pkgconfig:$PKG_CONFIG_PATH"
          '';
        };
      }
    );
}
