{ inputs, ... }:

{
  perSystem =
    { pkgs, ... }:
    let
      runtimeLibs = with pkgs; [
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

      runtime = pkgs.rustPlatform.buildRustPackage {
        pname = "scal-runtime";
        version = "0.1.0";

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

        buildInputs = runtimeLibs;
      };

    in
    {
      packages = {
        default = runtime;
        runtime = runtime;
      };

      devShells.default = pkgs.mkShell {
        packages =
          with pkgs;
          [
            rust-analyzer
            rustfmt
            clippy
            cargo
            rustc

            pkg-config

            clang
            llvmPackages.libclang

            alsa-lib.dev
          ]
          ++ runtimeLibs;

        shellHook = ''
          export LIBCLANG_PATH="${pkgs.llvmPackages.libclang.lib}/lib"

          export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath runtimeLibs}:$LD_LIBRARY_PATH"

          export PKG_CONFIG_PATH="${pkgs.alsa-lib.dev}/lib/pkgconfig:$PKG_CONFIG_PATH"
        '';
      };
    };
}
