{ inputs =
    { cargo2nix.url = "github:cargo2nix/cargo2nix";
      nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
      rust-overlay.url = "github:oxalica/rust-overlay";
      flake-utils.url = "github:numtide/flake-utils";
    };

  outputs = { cargo2nix, flake-utils, nixpkgs, rust-overlay, ... }:
    with builtins;
    flake-utils.lib.eachDefaultSystem
      (system:
         let
           pkgs =
             import nixpkgs
               { overlays =
                   [ (import "${cargo2nix}/overlay")
                     rust-overlay.overlay
                   ];

                 inherit system;
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
         { devShell =
             pkgs.mkShell
               { buildInputs =
                   with pkgs;
                   [ cargo
                     cargo2nix.defaultPackage.${system}
                     expat
                     freetype
                   ];
               };

           packages = mapAttrs (_: v: v {}) rustPkgs.workspace;
         }
      );
}
