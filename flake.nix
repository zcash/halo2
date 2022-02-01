{ inputs =
    { cargo2nix.url = "github:cargo2nix/cargo2nix";
      nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
      rust-overlay.url = "github:oxalica/rust-overlay";
      utils.url = "github:ursi/flake-utils/6";
    };

  outputs = { cargo2nix, nixpkgs, rust-overlay, utils, ... }@inputs:
    utils.for-default-systems
      ({ system, ... }:
         let
           pkgs =
             import nixpkgs
               { inherit system;

                 overlays =
                   [ (import "${cargo2nix}/overlay")
                     rust-overlay.overlay
                   ];
               };

             rustPkgs =
               pkgs.rustBuilder.makePackageSet'
                 { rustChannel = "1.56.1";
                   packageFun = import ./Cargo.nix;
                   packageOverrides =
                     let
                       expat-sys = pkgs.rustBuilder.rustLib.makeOverride {
                         name = "expat-sys";
                         overrideAttrs = drv: {
                           propagatedBuildInputs = drv.propagatedBuildInputs or [ ] ++ [ pkgs.expat ];
                         };
                       };
                       freetype-sys = pkgs.rustBuilder.rustLib.makeOverride {
                         name = "freetype-sys";
                         overrideAttrs = drv: {
                           propagatedBuildInputs = drv.propagatedBuildInputs or [ ] ++ [ pkgs.freetype ];
                         };
                       };
                    in
                    pkgs: pkgs.rustBuilder.overrides.all ++ [ expat-sys freetype-sys ];
                 };
           thing = (rustPkgs.workspace.halo2_proofs {});
         in
         { inherit rustPkgs;
           defaultPackage =
             thing;

           packages.example =
             pkgs.stdenv.mkDerivation
               { name = "circuit-layout";
                 src = ./.;
                 buildPhase =
                   ''
                     ${thing.runCargo} run --example circuit-layout --features="dev-graph"
                   '';
                 installPhase =
                   ''
                   mkdir $out
                   ln -s layout.png $out
                   '';
               };

           devShell =
             pkgs.mkShell
               { buildInputs =
                   with pkgs;
                   [ cargo
                     cargo2nix
                     gcc
                     rustc
                     cmake
                     expat
                     freetype
                     openssl
                     pkg-config
                     fontconfig
                   ];
               };
         }
      )
      inputs;
}
