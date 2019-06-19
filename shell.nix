with import <nixpkgs> {};
pkgs.mkShell {
  buildInputs = [
      rustup
      inkscape # needed for prettyprinting
      wireshark # for tshark
    ];
}
