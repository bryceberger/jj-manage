{
  inputs = {
    nixpkgs.url = "nixpkgs";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    fenix,
  }: let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};
    toolchain = fenix.packages.${system}.complete.toolchain;
  in {
    devShell.${system} = pkgs.mkShell {
      name = "jj-manage";
      packages = with pkgs; [
        toolchain
        cargo-nextest
      ];
    };
  };
}
