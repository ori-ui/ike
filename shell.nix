{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell rec {
  buildInputs = [
    pkgs.libGL
    pkgs.wayland
    pkgs.libxkbcommon
    pkgs.freetype
    pkgs.fontconfig
  ];

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
}
