# Copyright (c) 2025 Douglas Reis.
# Licensed under the Apache License, Version 2.0, see LICENSE for details.
# SPDX-License-Identifier: Apache-2.0

{rustPlatform}:
rustPlatform.buildRustPackage rec {
  pname = "zermio-cli";
  version = "0.1.0";
  src = ./.;
  cargoLock = {
    lockFile = "${src}/Cargo.lock";
  };

  doCheck = false;
  meta = {
    description = "A CLI tool to generate mmio code from hardware descripition languages";
    homepage = "https://github.com/engdoreis/zermio";
  };
}
