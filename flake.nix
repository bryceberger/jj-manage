{
  inputs = {
    nixpkgs.url = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    fenix,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
      toolchain = fenix.packages.${system}.complete.toolchain;
      ourRustPlatform = pkgs.makeRustPlatform {
        rustc = toolchain;
        cargo = toolchain;
      };
    in {
      packages.jujutsu-manage = ourRustPlatform.buildRustPackage {
        pname = "jujutsu-manage";
        version = "unstable-${self.shortRev or "dirty"}";
        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;
      };
      packages.default = self.packages.${system}.jujutsu-manage;

      devShell = pkgs.mkShell {
        name = "jj-manage";
        packages = with pkgs; [
          toolchain
          cargo-nextest
        ];
      };
    });
}
