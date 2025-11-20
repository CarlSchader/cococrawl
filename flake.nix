{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = inputs: with inputs;
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        craneLib = crane.mkLib pkgs;
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [ 
            pre-commit
            cargo
            convco
          ];
        };

        packages = {
          default = craneLib.buildPackage {
            src = craneLib.cleanCargoSource ./.;
          };
        };
      }
    );
}
