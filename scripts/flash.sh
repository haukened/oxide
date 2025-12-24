#!/bin/bash
set -euo pipefail

# check for root
if [ "$EUID" -ne 0 ]; then
  echo "Please run as root"
  exit
fi

# ensure /dev/sdb exists
if [ ! -b /dev/sdb ]; then
  echo "/dev/sdb not found. Please insert the USB drive."
  exit
fi

# ensure /dev/sdb1 is mounted
if ! mount | grep -q '/dev/sdb1'; then
  mount /dev/sdb1 /mnt
fi

# copy the loader.efi to the EFI partition
cp target/x86_64-unknown-uefi/release/loader.efi /mnt/EFI/BOOT/BOOTX64.EFI
sync

# unmount the EFI partition
umount /mnt

echo "Flashing completed successfully."