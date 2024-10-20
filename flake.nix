{
  description = "Build system and devshell environment for rust implementation of cognition";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs, ... }@inputs:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in
      {
        devShells.${system}.default = pkgs.mkShell
          {
            packages = with pkgs; [ rustc cargo binutils ];
            shellHook = ''
alias c="clear"
alias l="ls -la"
alias nr="nix run"
alias nb="nix build"
'';
          };
        defaultPackage.x86_64-linux = pkgs.stdenv.mkDerivation {
          name = "crank";
          src = "${self}";
          phases = [
            "buildPhase"
            "installPhase"
          ];
          buildPhase = with pkgs; ''
        CARGO_TARGET_DIR=$out ${pkgs.cargo}/bin/cargo build --manifest-path $src/Cargo.toml --release --locked
        '';
          installPhase = with pkgs; ''
mkdir -p $out/bin
cp $out/release/cognition $out/bin/cognition
'';
        };
      };
}
