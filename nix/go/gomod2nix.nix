{
  inputs,
  pkgs,
  lib,
  ...
}:
{
  packages = with pkgs; [
    gomod2nix
  ];

  overlays = [
    inputs.gomod2nix.overlays.default
  ];

  scripts = {
    update-gomod2nix = {
      description = "update gomod2nix.toml file";
      exec = ''
        for file in "$@"; do
          dir=$(dirname "$file")
          ${pkgs.gomod2nix}/bin/gomod2nix generate --dir "$dir"
          # format the generated file
          ${lib.getExe pkgs.taplo} fmt "$dir/gomod2nix.toml"
        done
      '';
    };
  };

  git-hooks.hooks = {
    update-gomod2nix = {
      enable = true;
      entry = "update-gomod2nix";
      files = "go.mod$";
    };
  };
}
