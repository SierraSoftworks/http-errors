{
  description = "A lightweight web server designed to serve error pages on demand.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, crane, flake-utils, advisory-db, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        inherit (pkgs) lib stdenv;

        craneLib = crane.lib.${system};
        src = pkgs.nix-gitignore.gitignoreSourcePure ''
          /target
          /result
          /.direnv
          *.nix
          '' ./.;

        nativeBuildInputs = []
        ++ lib.optionals stdenv.isDarwin [
          pkgs.libiconv
          pkgs.darwin.apple_sdk.frameworks.Security
          pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
        ];

        buildInputs = [
          pkgs.pkg-config
          pkgs.openssl
          pkgs.protobuf
        ];

        cargoArtifacts = craneLib.buildDepsOnly {
          inherit src nativeBuildInputs buildInputs;
        };

        http_errors = craneLib.buildPackage {
          inherit cargoArtifacts src nativeBuildInputs buildInputs;

          doCheck = false;
        };
      in
      {
        checks = {
          # Build the crate as part of `nix flake check` for convenience
          inherit http_errors;

          # Run clippy (and deny all warnings) on the crate source,
          # again, resuing the dependency artifacts from above.
          #
          # Note that this is done as a separate derivation so that
          # we can block the CI if there are issues here, but not
          # prevent downstream consumers from building our crate by itself.
          http_errors-clippy = craneLib.cargoClippy {
            inherit cargoArtifacts src buildInputs;
            cargoClippyExtraArgs = "--all-targets --no-deps";
          };

          http_errors-doc = craneLib.cargoDoc {
            inherit cargoArtifacts src;
          };

          # Check formatting
          http_errors-fmt = craneLib.cargoFmt {
            inherit src;
          };

          # Audit dependencies
          http_errors-audit = craneLib.cargoAudit {
            inherit src advisory-db;
          };

          # Run tests with cargo-nextest
          # Consider setting `doCheck = false` on `my-crate` if you do not want
          # the tests to run twice
          http_errors-nextest = craneLib.cargoNextest {
            inherit cargoArtifacts src nativeBuildInputs buildInputs;
            partitions = 1;
            partitionType = "count";

            # Disable impure tests (which access the network and/or filesystem)
            cargoNextestExtraArgs = "--no-fail-fast --features pure_tests";
          };
        };

        packages.default = http_errors;

        apps.default = flake-utils.lib.mkApp {
          drv = http_errors;
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = builtins.attrValues self.checks;

          # Extra inputs can be added here
          nativeBuildInputs = with pkgs; [
            cargo
            rustc
            nodejs
          ] ++ nativeBuildInputs;
        };
      });
}