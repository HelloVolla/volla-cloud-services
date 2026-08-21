{
  description = "android-service-runtime — development environment";

  # We depend on holonix only for the Holochain toolchain (holochain, hc,
  # lair-keystore, bootstrap-srv) and the rust-overlay it already pins.
  # Everything else (Rust toolchain, Android SDK/NDK, Tauri desktop libs) is
  # composed here from nixpkgs, so the repo has no dependency on any external
  # Tauri/Holochain dev-shell flake. (The previous dev shell came from
  # darksoil-studio/tauri-plugin-holochain, which has no holochain-0.7 branch.)
  inputs = {
    holonix.url = "github:holochain/holonix/main-0.7";

    nixpkgs.follows = "holonix/nixpkgs";
    flake-parts.follows = "holonix/flake-parts";
    rust-overlay.follows = "holonix/rust-overlay";
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = builtins.attrNames inputs.holonix.devShells;
      perSystem = { system, inputs', ... }:
        let
          pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ (import inputs.rust-overlay) ];
            config = {
              allowUnfree = true; # Android SDK/NDK are unfree
              android_sdk.accept_license = true;
            };
          };

          # Rust toolchain. Channel, components, and cross-compilation targets all
          # come from ./rust-toolchain.toml so nix and rustup users stay in sync.
          rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

          # Android SDK + NDK, matching the gradle configs in src-tauri/gen/*
          # (compileSdk 34, targetSdk 34, minSdk 27). The NDK is what `cargo ndk`
          # uses to cross-compile the Rust crates into jniLibs. 28.0.13004108 is
          # the NDK the previous (darksoil) dev shell shipped, so produced .so
          # files stay the same (clang 19, 16 KB page-aligned).
          ndkVersion = "28.0.13004108";
          androidComposition = pkgs.androidenv.composeAndroidPackages {
            # 34: targetSdk and compileSdk. Gradle cannot auto-install
            # platforms or build-tools into the read-only nix store, so every
            # version the build touches must be listed here.
            platformVersions = [ "34" ];
            buildToolsVersions = [ "34.0.0" ];
            includeNDK = true;
            ndkVersions = [ ndkVersion ];
            cmakeVersions = [ "3.22.1" ];
            includeEmulator = false; # emulator is provided by CI / installed on demand
            includeSystemImages = false;
          };
          androidSdk = androidComposition.androidsdk;
          androidHome = "${androidSdk}/libexec/android-sdk";
          ndkHome = "${androidHome}/ndk/${ndkVersion}";
        in
        {
          devShells.default = pkgs.mkShell {
            packages = (with inputs'.holonix.packages; [
              holochain
              hc
              lair-keystore
              bootstrap-srv
            ]) ++ [
              rust
            ] ++ (with pkgs; [
              nodejs_22 # pnpm
              pnpm
              cmake # aws-lc-sys (iroh/rustls crypto) builds its C sources with CMake
              pkg-config
              binaryen # wasm-opt, for building hApp/zome wasm
              shared-mime-info
              gsettings-desktop-schemas
              cargo-ndk # build Rust -> Android jniLibs
              jdk17 # Gradle
              androidSdk
            ]);

            shellHook = ''
              # getrandom 0.3 (in the zome dependency tree) refuses to build for
              # wasm32-unknown-unknown unless a backend is chosen; zome wasm never
              # calls OS randomness, so select the "custom" backend.
              export CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUSTFLAGS='--cfg getrandom_backend="custom"'

              # GTK schema lookup for the nix webkit; GSETTINGS_SCHEMAS_PATH is
              # filled by the glib setup hook from the schemas in `packages`.
              export XDG_DATA_DIRS=$GSETTINGS_SCHEMAS_PATH:$XDG_DATA_DIRS

              export ANDROID_HOME="${androidHome}"
              export ANDROID_SDK_ROOT="${androidHome}"
              export ANDROID_NDK="${ndkHome}"
              export ANDROID_NDK_ROOT="${ndkHome}"
              export ANDROID_NDK_HOME="${ndkHome}"
              export NDK_HOME="${ndkHome}"

              # cargo-ndk exports plain CC/CXX/AR pointing at the NDK clang, which
              # also hijacks *host* compiles (build scripts, proc-macro deps). The
              # HOST_* variants take precedence in the `cc` crate for host-targeted
              # units, so host builds keep the host toolchain even under
              # `cargo ndk`.
              export HOST_CC=gcc
              export HOST_CXX=g++
              export HOST_AR=ar

              export PS1='\[\033[1;35m\][asr-dev:\w]\$\[\033[0m\] '
            '';
          };
        };
    };
}
