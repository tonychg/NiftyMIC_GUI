{
  description = "NiftyMIC GUI v2";

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
        pkgs = nixpkgs.legacyPackages.${system};
        inherit (pkgs) lib;

        craneLib = crane.lib.${system};

        niftymic-gui = craneLib.buildPackage {
          src = lib.cleanSourceWith {
            src = craneLib.path ./.;
          };

          strictDeps = true;

          nativeBuildInputs = [
            pkgs.pkg-config
          ];
          buildInputs = with pkgs; [
            # Add additional build inputs here
            openssl
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            # Additional darwin specific inputs can be set here
            pkgs.libiconv
          ];

          # Additional environment variables can be set directly
        };
      in
      with pkgs;
      {
        checks = {
          inherit niftymic-gui;
        };

        packages = {
          default = niftymic-gui;
        };

        apps.default = flake-utils.lib.mkApp {
          drv = niftymic-gui;
        };

        devShells.default = craneLib.devShell {
          # Inherit inputs from checks.
          checks = self.checks.${system};

          # Extra inputs can be added here; cargo and rustc are provided by default.
          packages = [
            rust-analyzer
            git
            go-task
            xmedcon
            dcm2niix
            sops
          ];
        };
      });
}
