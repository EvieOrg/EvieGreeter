# To use this, you just need to add these.
### On flake.nix
in the inputs
```nix
  evie-greeter.url = "github:loyaall/evie-greeter";
```
on the outputs you just need to type evie-greeter in the list as you have

### On configuration.nix
in the imports
```nix
  inputs.evie-greeter.nixosModules.evie-greeter
```
then to use it, you need to copy also that (primary monitor is where you want the login card, in other displays it just let you see the clock)
```nix
  programs.evie-greeter = {
    enable = true;
    primaryMonitor = "DP-3";
  };

  services.greetd = {
    enable = true;
    settings.default_session = {
      command = "${config.programs.evie-greeter.package}/bin/evie-greeter-session";
      user = "greeter";
    };
  };
```
# That's it!
