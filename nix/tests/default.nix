{
  self,
  pkgs,
  ...
}: {
  assets = import ./test-assets.nix {inherit pkgs self;};
}

