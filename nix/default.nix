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
    #buildPlatform = "crate2nix";
    defaultOutputs = { app = "zellij"; package = "zellij"; };
     #overrides = {
        #crateOverrides = common: prev:
            #let
                #pkgs = common.pkgs;
                #lib  = common.lib;
            #in
            #{
            #zellij = _prev: {
            #nativeBuildInputs = [ pkgs.cargo ];
            #BuildInputs = [ pkgs.cargo ];
            #};
        #};
        #};
    }

#}
#//
## native nix
#import ./zellij.nix { inherit nixpkgs rust-overlay flake-utils crate2nix;}
