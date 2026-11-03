{
  lib,
  rustPlatform,
  fetchPnpmDeps,
  pnpmConfigHook,
  pnpm_11,
  nodejs_24,
  cargo-tauri,
  pkg-config,
  protobuf,
  wrapGAppsHook3,
  webkitgtk_4_1,
  libayatana-appindicator,
  librsvg,
  openssl,
  xdotool,
  glib-networking,
}:

rustPlatform.buildRustPackage (finalAttrs: {
  pname = "deadlock-plus";
  version = (lib.importJSON ../apps/desktop/package.json).version;

  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.difference ../. (
      lib.fileset.unions [
        ../nix
        ../flake.nix
        ../flake.lock
      ]
    );
  };

  cargoHash = "sha256-zGg/EllgJoDrEG3lrX3/pOc5fNVEyMifQ3b+KCEOJXI=";

  pnpmWorkspaces = [ "deadlock-plus..." ];
  pnpmDeps = fetchPnpmDeps {
    inherit (finalAttrs)
      pname
      version
      src
      pnpmWorkspaces
      ;
    pnpm = pnpm_11;
    fetcherVersion = 5;
    hash = "sha256-kdYwHSXGqY/5smixRlD/J4ZOP261jiKpeOuz17tLo0Y=";
  };

  nativeBuildInputs = [
    cargo-tauri.hook
    pnpmConfigHook
    pnpm_11
    nodejs_24
    pkg-config
    protobuf
    wrapGAppsHook3
  ];

  buildInputs = [
    webkitgtk_4_1
    libayatana-appindicator
    librsvg
    openssl
    xdotool
    glib-networking
  ];

  strictDeps = true;

  buildAndTestSubdir = "apps/desktop";

  # Same scope as CI; the default test run also builds examples, and net_probe is Windows-only.
  cargoTestFlags = [
    "--workspace"
    "--lib"
  ];

  checkFlags = [
    # Spawns `sleep` with a fake argv[0]; the sandbox's multicall coreutils dispatches on argv[0] and exits.
    "--skip=tests::scan_system_recognizes_a_game_argv_zero_with_a_different_process_name"
  ];

  # The Vulkan frame layer needs a zig build against glibc 2.31 to load in Steam's runtime container, so it is
  # left out: null removes the deb's extra files entry, since Tauri merges --config as a JSON merge patch.
  # Updates come from Nix, not from signed updater bundles.
  tauriBuildFlags = [
    "--config"
    (builtins.toJSON {
      bundle = {
        createUpdaterArtifacts = false;
        linux.deb.files = null;
      };
    })
  ];

  # The tray library is loaded with dlopen, so it never shows up as a link dependency.
  postFixup = ''
    patchelf --add-rpath ${
      lib.makeLibraryPath [ libayatana-appindicator ]
    } $out/bin/.deadlock-plus-wrapped
  '';

  meta = {
    description = "Companion app for Deadlock: Rich Presence, server picker, live match stats and match history";
    homepage = "https://deadlockplus.app";
    changelog = "https://github.com/deadlock-plus/deadlock-plus/blob/v${finalAttrs.version}/CHANGELOG.md";
    license = lib.licenses.gpl3Plus;
    mainProgram = "deadlock-plus";
    platforms = [ "x86_64-linux" ];
    maintainers = [ ];
  };
})
