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
    pkgs.cargo-insta
    pkgs.sqlite
    pkgs.systemfd
    pkgs.watchexec
    pkgs.litestream
  ];

  # See full reference at https://devenv.sh/reference/options/
}
