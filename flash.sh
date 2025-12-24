#!/bin/bash
set -euo pipefail

# check root
if [ "$EUID" -ne 0 ]; then
    echo "Please run as root"
    exit 1
fi

# check /dev/sdb1 exists
if [ ! -b /dev/sdb1 ]; then
  echo "/dev/sdb1 does not exist"
  exit 1
fi

# make sure it's unmounted
umount /dev/sdb1 > /dev/null 2>&1 || true

# mount it
mount /dev/sdb1 /mnt

# ensure /mnt/EFI/BOOT exists
mkdir -p /mnt/EFI/BOOT

# copy the efi from target
cp target/x86_64-unknown-uefi/release/oxide.efi /mnt/EFI/BOOT/BOOTX64.EFI

# sync to ensure all writes are done
sync

# unmount
umount /mnt

echo "Flashed /dev/sdb1 successfully."