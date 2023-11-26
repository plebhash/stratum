{
  description = "StratumV2 Reference Implementation (SRI)";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        # todo: take config files from repo tree and place on package tree

        craneLib = crane.lib.${system};
        sri = craneLib.buildPackage {
          src = craneLib.cleanCargoSource (craneLib.path ./.);
          strictDeps = true;
          doCheck = false;

          buildInputs = [
            pkgs.pkg-config
          ];
        };
      in
      {
        checks = {
          inherit sri;
        };

        packages.default = sri;

        apps.default = flake-utils.lib.mkApp {
          drv = sri;
        };

        apps."<system>"."jd-server" = {
          type = "jd-server";
          program = "bin/jd_server";
        };

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};

          packages = [];
        };
      });
}