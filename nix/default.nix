{ self
, nixpkgs
, rust-overlay
, flake-utils
, flake-compat
#, crate2nix
, nixCargoIntegration
}:

nixCargoIntegration.lib.makeOutputs {
    root = ../.;
    #defaultOutputs = { app = "zellij"; package = "zellij"; };
    }

#}
#//
## native nix
#import ./zellij.nix { inherit nixpkgs rust-overlay flake-utils crate2nix;}
