{ inputs =
    { nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
      utils.url = "github:ursi/flake-utils/6";
    };

  outputs = { utils, ... }@inputs:
    utils.for-default-systems
      ({ pkgs, ... }:
         { devShell =
             pkgs.mkShell
               { buildInputs =
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
