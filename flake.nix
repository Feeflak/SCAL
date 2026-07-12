{
  description = "SCAL";

  imports = [ ];

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

        src = ./scal-runtime;

        cargoLock.lockFile = ./Cargo.lock;

        nativeBuildInputs = [
          pkgs.pkg-config
          pkgs.clang
        ];

        buildInputs = runtimeLibs;
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
        packages = [
          pkgs.rust-analyzer
          pkgs.rustfmt
          pkgs.clippy
          pkgs.cargo
          pkgs.rustc

          pkgs.pkg-config
          pkgs.clang
          pkgs.llvmPackages.libclang
          pkgs.alsa-lib.dev
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
