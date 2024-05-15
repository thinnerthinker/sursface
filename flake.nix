{
  description = "Sursface, a cross-platform rendering library";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, crane, fenix, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        mkPlatform = { system, arch, depsBuild ? [ ], env ? { }, postInstall ? _: "", isDefault ? false }: {
          name = arch;
          value = let pi = postInstall; in
            rec {
              inherit system;
              inherit arch;
              inherit depsBuild;
              inherit env;
              inherit isDefault;
              postInstall = crateName: if isDefault then "" else pi crateName;
            };
        };

        mkCraneLib = targetPlatform:
          let
            toolchain = with fenix.packages.${system};
              combine [
                latest.rustc
                latest.cargo
                targets.${targetPlatform.system}.latest.rust-std
              ];
          in
          (crane.mkLib pkgs).overrideToolchain toolchain;

        targetPlatforms = builtins.listToAttrs (map mkPlatform [
          {
            system = "x86_64-unknown-linux-gnu";
            arch = "x86_64-linux";
            depsBuild = with pkgs; [ patchelf ];
            postInstall = crateName: ''
                find $out -type f -exec sh -c '
                if file "$1" | grep -q "ELF .* executable"; then
                  patchelf --set-interpreter "/lib64/ld-linux-x86-64.so.2" "$1"
                fi
              ' sh {} \;
            '';
            env = {
              LD_LIBRARY_PATH = with pkgs; lib.makeLibraryPath [
                libxkbcommon
                wayland
                xorg.libX11
                xorg.libXrandr
                xorg.libXrender
                xorg.libXcursor
                xorg.libxcb
                xorg.libXi

                libGL
                vulkan-loader
              ];
            };
          }
          {
            system = "x86_64-pc-windows-gnu";
            arch = "x86_64-windows";
            depsBuild = with pkgs.pkgsCross; [
              mingwW64.stdenv.cc
              mingwW64.windows.pthreads
            ];
            env = {
              # fixes issues related to libring
              TARGET_CC = with pkgs.pkgsCross; "${mingwW64.stdenv.cc}/bin/${mingwW64.stdenv.cc.targetPrefix}cc";

              #fixes issues related to openssl
              OPENSSL_DIR = "${pkgs.openssl.dev}";
              OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
              OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include/";
            };
          }
          {
            system = "wasm32-unknown-unknown";
            arch = "wasm32-unknown";
            depsBuild = with pkgs; [ wasm-bindgen-cli ];
            postInstall = crateName: ''
              mkdir $out/bindgen
              find $out/bin -type f -name "*.wasm" -exec wasm-bindgen {} --out-dir $out/bindgen --web \;
            '';
            env = {
              packages = with pkgs; [ wasm-bindgen-cli ];
            };
          }
        ]);

        crateFor = srcLocation: targetPlatform:
          let
            craneLib = mkCraneLib targetPlatform;
          in
          craneLib.buildPackage
            ({
              src = craneLib.cleanCargoSource (craneLib.path srcLocation);
              cargoVendorDir = craneLib.vendorCargoDeps { cargoLock = ./Cargo.lock; };

              strictDeps = true;
              doCheck = false;

              CARGO_BUILD_TARGET = targetPlatform.system;
              depsBuildBuild = targetPlatform.depsBuild;

              postInstall = targetPlatform.postInstall (craneLib.crateNameFromCargoToml { cargoToml = "${srcLocation}/Cargo.toml"; }).pname;
            } // targetPlatform.env);

        shellFor = srcLocation: targetPlatform:
          let
            craneLib = mkCraneLib targetPlatform;
          in
          craneLib.devShell ({
            CARGO_BUILD_TARGET = targetPlatform.system;
            depsBuildBuild = targetPlatform.depsBuild;
          } // targetPlatform.env);

        eachPlatform = targetPlatforms: mkFor: pkgs.lib.attrsets.mapAttrs (name: platform: mkFor platform) targetPlatforms // {
          default = mkFor ((mkPlatform (targetPlatforms.${system} // { isDefault = true; })).value);
        };
      in
      rec {
        packagesForEachPlatform = srcLocation: eachPlatform targetPlatforms (crateFor srcLocation);
        devShellsForEachPlatform = srcLocation: eachPlatform targetPlatforms (shellFor srcLocation);

        packages = packagesForEachPlatform ./.;
        devShells = devShellsForEachPlatform ./.;
      }
    );
} 
 