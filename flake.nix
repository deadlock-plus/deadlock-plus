{
  description = "Deadlock+, a companion app for Deadlock";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
      deadlock-plus = pkgs.callPackage ./nix/package.nix { };
    in
    {
      packages.${system} = {
        inherit deadlock-plus;
        default = deadlock-plus;
      };

      overlays.default = final: prev: { deadlock-plus = final.callPackage ./nix/package.nix { }; };

      devShells.${system}.default = pkgs.mkShell {
        inputsFrom = [ deadlock-plus ];
        packages = [ pkgs.rust-analyzer ];
      };
    };
}
