{
  pkgs,
  lib,
  config,
  ...
}:
{
  # https://devenv.sh/languages/
  languages.rust = {
    enable = true;
    channel = "stable";
  };

  packages = [
    pkgs.watchexec
  ];

  # See full reference at https://devenv.sh/reference/options/
}
