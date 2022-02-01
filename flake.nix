{ inputs =
    { make-shell.url = "github:ursi/nix-make-shell/1";
      nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
      utils.url = "github:ursi/flake-utils/6";
    };

  outputs = { utils, ... }@inputs:
    utils.for-default-systems
      ({ make-shell, pkgs, ... }:
         { devShell =
             pkgs.mkShell
               { nativeBuildInputs =
                   with pkgs;
                   [ cargo
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
