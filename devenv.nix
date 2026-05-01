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

  git-hooks.hooks = {
    rustfmt.enable = true;
    clippy.enable = true;
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
