{
  description = "dogonline-companion dev shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { nixpkgs, rust-overlay, ... }:
    let
      systems = [ "x86_64-linux" ];
      forAllSystems =
        f:
        nixpkgs.lib.genAttrs systems (
          system:
          f (
            import nixpkgs {
              inherit system;
              overlays = [ rust-overlay.overlays.default ];
            }
          )
        );
    in
    {
      devShells = forAllSystems (
        pkgs:
        let
          # move main to the given rev (default @-), then push
          push = pkgs.writeShellApplication {
            name = "push";
            runtimeInputs = [ pkgs.jujutsu ];
            text = ''
              rev="''${1:-@-}"
              jj bookmark set main -r "$rev"
              jj git push --bookmark main
            '';
          };
        in
        {
          default = pkgs.mkShell {
            packages = [
              # toolchain + components come from rust-toolchain.toml
              (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
              pkgs.trunk
              push
            ];
          };
        }
      );
    };
}
