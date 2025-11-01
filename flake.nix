{
  description = "Chronomatrix - Rust GTK4/libadwaita GUI application";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        # Common libraries needed for GTK4/libadwaita development
        buildInputs = with pkgs; [
          gtk4
          libadwaita
          glib
          cairo
          pango
          gdk-pixbuf
          graphene
        ];

        # Native build inputs for compilation
        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
          wrapGAppsHook4
          desktop-file-utils
          meson
          ninja
        ];

      in
      {
        devShells.default = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs;

          # Environment variables for development
          shellHook = ''
            echo "Chronomatrix - Rust Development Environment"
            echo "Rust version: $(rustc --version)"
            echo "GTK4 version: ${pkgs.gtk4.version}"
            echo "libadwaita version: ${pkgs.libadwaita.version}"
            echo ""
            echo "Ready to build!"

            # Set PKG_CONFIG_PATH to help cargo find libraries
            export PKG_CONFIG_PATH="${pkgs.glib.dev}/lib/pkgconfig:${pkgs.gtk4.dev}/lib/pkgconfig:${pkgs.libadwaita.dev}/lib/pkgconfig:$PKG_CONFIG_PATH"

            # Help cargo find libraries at runtime
            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath buildInputs}:$LD_LIBRARY_PATH"

            # GSettings schemas location
            export XDG_DATA_DIRS="${pkgs.gtk4}/share/gsettings-schemas/${pkgs.gtk4.name}:${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:$XDG_DATA_DIRS"
          '';
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "chronomatrix";
          version = "0.1.0";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          inherit buildInputs nativeBuildInputs;

          meta = with pkgs.lib; {
            description = "Digital clock where each digit is made of analog clocks";
            homepage = "https://github.com/yourusername/chronomatrix";
            license = licenses.mit;
            maintainers = [ ];
          };
        };
      }
    );
}
