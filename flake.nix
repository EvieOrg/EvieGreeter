{
  description = "Evie Greeter – greetd greeter in Rust + GTK4, coerente con Evie shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    let
      # ── NixOS module: programs.evie-greeter.enable ───────────────────
      # Stesso pattern di Evie — installa il binario nel profilo di sistema.
      # La configurazione di greetd va fatta nel tuo greeter.nix usando
      # inputs.evie-greeter.packages.${system}.evie-greeter direttamente.
      nixosModule = { config, lib, pkgs, ... }:
        let
          cfg = config.programs.evie-greeter;
          greeterPkg = self.packages.${pkgs.stdenv.hostPlatform.system}.evie-greeter;
        in {
          options.programs.evie-greeter = {
            enable = lib.mkEnableOption "Evie Greeter – installa il binario nel sistema";

            package = lib.mkOption {
              type = lib.types.package;
              default = greeterPkg;
              defaultText = lib.literalExpression "evie-greeter.packages.\${system}.evie-greeter";
              description = "Il pacchetto evie-greeter da usare.";
            };

            primaryMonitor = lib.mkOption {
              type = lib.types.nullOr lib.types.str;
              default = null;
              example = "DP-3";
              description = "Connettore del monitor su cui mostrare la login card (es. \"DP-3\", \"HDMI-A-1\"). Se null usa il primo monitor disponibile.";
            };
          };

          config = lib.mkIf cfg.enable {
            environment.systemPackages = [ cfg.package pkgs.bibata-cursors ];

            # Scrive /etc/greeter/config.toml se primaryMonitor è specificato
            environment.etc."greeter/config.toml" = lib.mkIf (cfg.primaryMonitor != null) {
              text = ''
                primary_monitor = "${cfg.primaryMonitor}"
              '';
            };

            # Genera /etc/greeter/hyprland.conf con monitor primario già configurato
            environment.etc."greeter/hyprland.conf" = {
              text = ''
                ${lib.readFile "${cfg.package}/share/evie-greeter/hyprland.conf"}
                ${lib.optionalString (cfg.primaryMonitor != null) ''
                monitor = ${cfg.primaryMonitor},preferred,0x0,1
                workspace = 1, monitor:${cfg.primaryMonitor}, default:true
                exec-once = bash -c 'sleep 0.5 && hyprctl dispatch movecursor 960 540'
                ''}
              '';
            };

            # ACL per permettere a greeter di leggere gli avatar utente
            system.activationScripts.greeterAvatarAcl = {
              deps = [ "users" ];
              text = ''
                for home in /home/*/; do
                  [ -d "$home" ] || continue
                  ${pkgs.acl}/bin/setfacl -m u:greeter:x "$home" 2>/dev/null || true
                  for f in "$home/.face" "$home/.face.icon"; do
                    [ -e "$f" ] && ${pkgs.acl}/bin/setfacl -m u:greeter:r "$f" 2>/dev/null || true
                  done
                done
                # AccountsService icons
                if [ -d /var/lib/AccountsService/icons ]; then
                  ${pkgs.acl}/bin/setfacl -m u:greeter:x /var/lib/AccountsService 2>/dev/null || true
                  ${pkgs.acl}/bin/setfacl -m u:greeter:x /var/lib/AccountsService/icons 2>/dev/null || true
                  ${pkgs.acl}/bin/setfacl -R -m u:greeter:r /var/lib/AccountsService/icons 2>/dev/null || true
                fi
              '';
            };
          };
        };

      # ── Home-manager module: programs.evie-greeter.enable ────────────
      # Stesso pattern di Evie — installa il binario nel profilo utente
      # (utile per sviluppo/test, non necessario in produzione)
      homeManagerModule = { config, lib, pkgs, ... }:
        let
          cfg = config.programs.evie-greeter;
          greeterPkg = self.packages.${pkgs.stdenv.hostPlatform.system}.evie-greeter;
        in {
          options.programs.evie-greeter = {
            enable = lib.mkEnableOption "Evie Greeter – installa il binario nel profilo";

            package = lib.mkOption {
              type = lib.types.package;
              default = greeterPkg;
              defaultText = lib.literalExpression "evie-greeter.packages.\${system}.evie-greeter";
              description = "Il pacchetto evie-greeter da usare.";
            };
          };

          config = lib.mkIf cfg.enable {
            home.packages = [ cfg.package ];
          };
        };

    in
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        # Rust toolchain: edition 2024 richiede ≥ 1.85 stable
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" ];
        };

        nativeBuildDeps = with pkgs; [
          rustToolchain
          pkg-config
          makeWrapper
        ];

        buildDeps = with pkgs; [
          # GTK4 + layer-shell + libadwaita
          gtk4
          gtk4-layer-shell
          libadwaita
          glib
          gdk-pixbuf
          graphene
          pango
          cairo

          # D-Bus (zbus, per AccountsService avatar)
          dbus

          # Wayland
          wayland
          wayland-protocols
          libxkbcommon

          # Immagini (crate image, usato indirettamente da gdk-pixbuf)
          libjpeg
          libpng
          zlib
        ];

        evie-greeter = pkgs.rustPlatform.buildRustPackage {
          pname = "evie-greeter";
          version = "0.1.0";

          src = builtins.path {
            path = ./.;
            name = "evie-greeter-source";
            filter = path: type:
              let base = baseNameOf path; in
              !(builtins.elem base [ ".git" "result" "target" ]);
          };

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = nativeBuildDeps;
          buildInputs = buildDeps;

          postInstall = ''
            # Sostituisce il placeholder con il path assoluto del binario
            substituteInPlace hyprland.conf \
              --replace "evie-greeter;" "$out/bin/evie-greeter;"

            # Installa il config Hyprland e il wallpaper nello share
            install -Dm644 hyprland.conf $out/share/evie-greeter/hyprland.conf
            install -Dm644 wallpaper $out/share/evie-greeter/wallpaper

            # Wrappa il binario con le lib necessarie a runtime
            wrapProgram $out/bin/evie-greeter \
              --prefix LD_LIBRARY_PATH : ${pkgs.lib.makeLibraryPath buildDeps}

            # Script di avvio: usa start-hyprland con il config generato da NixOS
            cat > $out/bin/evie-greeter-session << 'EOF'
#!/usr/bin/env bash
exec @hyprland@/bin/Hyprland --config /etc/greeter/hyprland.conf
EOF
            chmod +x $out/bin/evie-greeter-session
            substituteInPlace $out/bin/evie-greeter-session \
              --replace "@hyprland@" "${pkgs.hyprland}"
            wrapProgram $out/bin/evie-greeter-session \
              --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.dbus ]} \
              --prefix LD_LIBRARY_PATH : ${pkgs.lib.makeLibraryPath buildDeps}
          '';

          meta = with pkgs.lib; {
            description = "Evie Greeter – greetd greeter GTK4/Wayland";
            license = licenses.mit;
            maintainers = [ ];
            platforms = platforms.linux;
          };
        };
      in
      {
        packages = {
          inherit evie-greeter;
          default = evie-greeter;
        };

        apps.default = flake-utils.lib.mkApp {
          drv = evie-greeter;
          exePath = "/bin/evie-greeter";
        };

        devShells.default = pkgs.mkShell {
          name = "evie-greeter-dev";

          nativeBuildInputs = nativeBuildDeps;
          buildInputs = buildDeps;

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildDeps;

          shellHook = ''
            echo ""
            echo "  🌿 Evie Greeter dev shell"
            echo "  ─────────────────────────────────────"
            echo "  cargo build        – compila"
            echo "  cargo run          – avvia (richiede GREETD_SOCK)"
            echo "  cargo clippy       – linting"
            echo "  cargo fmt          – formattazione"
            echo ""
          '';
        };
      }
    ) // {
      nixosModules.default = nixosModule;
      nixosModules.evie-greeter = nixosModule;
      homeManagerModules.default = homeManagerModule;
      homeManagerModules.evie-greeter = homeManagerModule;
    };
}
