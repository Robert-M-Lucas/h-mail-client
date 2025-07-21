{
  description = "A Nix-flake-based Rust development environment";

  inputs = {
    nixpkgs = {
      type = "github";
      owner = "NixOS";
      repo = "nixpkgs";
      ref = "nixos-25.05";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forEachSupportedSystem = f: inputs.nixpkgs.lib.genAttrs supportedSystems (system: f {
        pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [
            inputs.rust-overlay.overlays.default
            inputs.self.overlays.default
          ];
        };
      });
    in
    {
      overlays.default = final: prev: {
        # rustToolchain =
        #   let
        #     rust = prev.rust-bin;
        #   in
        #   if builtins.pathExists ./rust-toolchain.toml then
        #     rust.fromRustupToolchainFile ./rust-toolchain.toml
        #   else if builtins.pathExists ./rust-toolchain then
        #     rust.fromRustupToolchainFile ./rust-toolchain
        #   else
        #     rust.stable.latest.default.override {
        #       extensions = [ "rust-src" "rustfmt" ];
        #     };
      };

      devShells = forEachSupportedSystem ({ pkgs }: {
        default = pkgs.mkShell {
          packages = with pkgs; [
            # rustToolchain
            openssl
            # pkg-config
            # cargo-deny
            # cargo-edit
            # cargo-watch
            # rust-analyzer
            # ...
          ];

          env = {
            OPENSSL_LIB_DIR="${pkgs.lib.getLib pkgs.openssl}/lib";
            OPENSSL_NO_VENDOR=1;
            OPENSSL_DIR="${pkgs.openssl.dev}";
            PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig:$PKG_CONFIG_PATH";
            # Required by rust-analyzer
            # RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
          };
        };
      });
    };
}
