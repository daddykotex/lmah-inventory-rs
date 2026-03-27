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
    pkgs.sqlite
    pkgs.cargo-insta
  ];

  # See full reference at https://devenv.sh/reference/options/
}
