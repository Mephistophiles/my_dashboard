{ pkgs, lib, config, inputs, ... }:

{
  # https://devenv.sh/basics/
  env.GREET = "my_dashboard";

  dotenv.enable = true;

  # https://devenv.sh/packages/
  packages = [ pkgs.git pkgs.just ];

  # https://devenv.sh/languages/
  languages.rust.enable = true;

  # https://devenv.sh/processes/
  # processes.cargo-watch.exec = "cargo-watch";

  # https://devenv.sh/services/
  # services.postgres.enable = true;

  # https://devenv.sh/tasks/
  # tasks = {
  #   "myproj:setup".exec = "mytool build";
  #   "devenv:enterShell".after = [ "myproj:setup" ];
  # };

  # https://devenv.sh/tests/
  enterTest = ''
    echo "Running tests"
    git --version | grep --color=auto "${pkgs.git.version}"
  '';

  # https://devenv.sh/git-hooks/
  git-hooks.hooks = {
    rustfmt = {
      enable = true;
    };
    clippy = {
      enable = true;
      settings = {
        allFeatures = true;
        denyWarnings = true;
      };
    };
  };

  # See full reference at https://devenv.sh/reference/options/
}
