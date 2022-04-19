{
  pkgs,
  self,
}: let
  system = "x86_64-linux";
  user = "alice";
in
  pkgs.nixosTest {
    name = "test-assets";
    enableOCR = true;
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
     users.users.${user} = {
      uid = 1000;
      isNormalUser = true;
    };
    };

    testScript = ''
    from shlex import quote

    def su(user, cmd):
        return f"su - {user} -c {quote(cmd)}"


    start_all()
    machine.wait_for_unit("default.target")
    machine.succeed(
      su('${user}', "${self.outputs.packages.${system}.zellij}/bin/zellij")
      )
    '';
  }

