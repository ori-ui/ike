{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell rec {
  buildInputs = [
    pkgs.pkg-config
    pkgs.wayland
    pkgs.libxkbcommon
    pkgs.freetype
    pkgs.fontconfig
    pkgs.vulkan-loader
    pkgs.vulkan-validation-layers
  ];

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
}
