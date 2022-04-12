{
  pkgs,
  self,
}: let
  system = "x86_64-linux";
in
  pkgs.nixosTest {
    name = "test-assets";
    nodes.machine = {
      config,
      pkgs,
      ...
    }: {
      virtualisation.graphics = false;
      boot.kernelModules = ["kvm-intel"];
      environment.variables = {
        SHELL = "/usr/bin/env bash";
      };
    };

    testScript = ''
      start_all()
      machine.wait_for_unit("default.target")
      machine.succeed(
        "${self.outputs.packages.${system}.zellij}/bin/zellij"
        )
      machine.succeed(
        "${self.outputs.packages.${system}.zellij}/bin/zellij ka"
        )
      machine.fail(
        "${self.outputs.packages.${system}.zellij}/bin/zellij ls"
        )
    '';
  }

