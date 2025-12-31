#!/bin/bash

set -euo pipefail

cargo build -r -p loader --target x86_64-unknown-uefi

sudo cp target/x86_64-unknown-uefi/release/loader.efi /srv/tftp/ipxe.efi