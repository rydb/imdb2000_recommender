{
  description = "Rust DevShells: Linux (wild linker)";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
    wild = {
      url = "github:wild-linker/wild";
      flake = false;
    };
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, wild, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [
          (import rust-overlay)
          (import wild)
        ];

        pkgs = import nixpkgs { inherit system overlays; };
        lib = pkgs.lib;

        mkBaseDeps = pkgs: with pkgs; [
          gcc
          pkg-config
          dioxus-cli
          binutils
          gnumake
          flatpak
          flatpak-builder
        ] ++ (with pkgs; [
          openssl
          glib
          pango
          gdk-pixbuf
          cairo
          atk
          gtk3
          webkitgtk_4_1
          xdotool
          zlib
          python3
          libxkbcommon
          vulkan-loader
          wayland
        ]);

        rustStable = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        wildStdenv = pkgs.useWildLinker pkgs.stdenv;
        mkShellWild = pkgs.mkShell.override { stdenv = wildStdenv; };

      in
      with pkgs;
      {
        devShells = {
          default = mkShellWild {
            buildInputs = (mkBaseDeps pkgs) ++ [ rustStable flatpak ];
            env.RUSTFLAGS = "-C link-arg=-Wl,-rpath,${lib.makeLibraryPath (with pkgs; [ libxkbcommon vulkan-loader wayland ])}";
            shellHook = ''
              echo "Ensuring flatpak Flathub remote and runtimes (user) ..."
              flatpak remote-add --if-not-exists --user flathub https://flathub.org/repo/flathub.flatpakrepo
              echo "Adding flatpak sdk"
              flatpak install --user --noninteractive flathub org.freedesktop.Platform//24.08 org.freedesktop.Sdk//24.08

              # Add ~/.cargo/bin to PATH for locally installed binaries
              export PATH="$HOME/.cargo/bin:$PATH"
            '';
          };
        };

      }
    ) // {
      nixConfig = {
        extra-substituters = [ "https://cache.nixos.org" ];
        extra-trusted-public-keys = [ "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY=" ];
      };
    };
}