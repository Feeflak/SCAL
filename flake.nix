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

        runtimeDeps = with pkgs; [
          ffmpeg

          wayland
          libxkbcommon

          libx11
          libxcursor
          libxi
          libxrandr
          libxcb

          alsa-lib
          libpulseaudio
        ];

        buildDeps = with pkgs; [
          pkg-config
          clang
          llvmPackages.libclang
        ];

        runtime = pkgs.rustPlatform.buildRustPackage {
          pname = "scal-runtime";
          version = "1.3.1";

          src = ./.;

          cargoLock.lockFile = ./Cargo.lock;

          cargoBuildFlags = [
            "-p"
            "scal-runtime"
          ];

          nativeBuildInputs = buildDeps;

          buildInputs = runtimeDeps;

          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

          PKG_CONFIG_PATH = pkgs.lib.makeSearchPath "lib/pkgconfig" [
            pkgs.alsa-lib.dev
            pkgs.ffmpeg
          ];

          # winit/alsa/etc. dlopen() wayland, xkbcommon, X11 and pulse at runtime,
          # so Nix doesn't add them to the RPATH automatically. Add them here
          # instead of relying on LD_LIBRARY_PATH.
          postFixup = ''
            patchelf \
              --add-rpath "${pkgs.lib.makeLibraryPath runtimeDeps}" \
              $out/bin/scal-runtime
          '';

          # Needed by some crates using bindgen
          BINDGEN_EXTRA_CLANG_ARGS = builtins.concatStringsSep " " [
            "-I${pkgs.llvmPackages.libclang.lib}/lib/clang/${pkgs.llvmPackages.libclang.version}/include"
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
          packages =
            buildDeps
            ++ runtimeDeps
            ++ (with pkgs; [
              rust-analyzer
              rustfmt
              clippy
              cargo
              rustc
            ]);

          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

          PKG_CONFIG_PATH = pkgs.lib.makeSearchPath "lib/pkgconfig" [
            pkgs.alsa-lib.dev
            pkgs.ffmpeg
          ];

          # Bake the runtime libs into binaries built by cargo (rpath) instead of
          # exporting LD_LIBRARY_PATH, which leaks into every program started in
          # the shell and breaks binaries built against a different nixpkgs.
          RUSTFLAGS = "-C link-arg=-Wl,--disable-new-dtags -C link-arg=-Wl,-rpath,${pkgs.lib.makeLibraryPath runtimeDeps}";
        };
      }
    );
}
