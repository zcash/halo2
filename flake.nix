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
         in
         { defaultPackage = rustPkgs.workspace.halo2_proofs {};

           devShell =
             pkgs.mkShell
               { buildInputs =
                   with pkgs;
                   # all of the packages marked with a # seem to make no difference in my ability to run `cargo run --example circuit-layout --features=dev-graph`
                   [ cargo
                     cargo2nix.defaultPackage.${system}
                     cmake
                     expat
                     fontconfig #
                     freetype
                     gcc #
                     openssl #
                     pkg-config #
                     rustc #
                   ];
               };
         }
      )
      inputs;
}
