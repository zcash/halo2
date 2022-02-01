{ inputs =
    { cargo2nix.url = "github:cargo2nix/cargo2nix";
      nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
      utils.url = "github:ursi/flake-utils/6";
    };

  outputs = { utils, ... }@inputs:
    utils.for-default-systems
      ({ cargo2nix, pkgs, ... }:
         { devShell =
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
