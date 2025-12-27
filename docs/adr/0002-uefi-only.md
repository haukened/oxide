# ADR 0002: UEFI-Only Boot Strategy

## Status
Accepted

## Context

Oxide targets modern x86_64 systems and is being developed at a time when UEFI
firmware is ubiquitous on contemporary hardware.

Legacy BIOS-based boot flows require:
- Starting the CPU in 16-bit real mode
- Manually transitioning through protected mode to long mode
- Reliance on legacy interrupt interfaces (e.g., INT 10h, INT 13h)
- Hardware assumptions that no longer reflect modern systems

Supporting legacy BIOS would introduce significant complexity, fragile
bootstrapping code, and historical constraints that provide no architectural
benefit to the long-term design of the operating system.

The project’s goals emphasize clarity, explicit ownership, and modern system
assumptions rather than maximal backward compatibility.

## Decision

Oxide will support **UEFI-only booting** on x86_64 systems.

The firmware is treated strictly as a bootstrap environment whose role is to:
- Enter 64-bit long mode
- Provide early access to standardized hardware discovery mechanisms
- Load the initial OS loader image

UEFI is exited as early as practical via `ExitBootServices`, after which the
operating system assumes full control of the machine.

Legacy BIOS boot paths, real-mode execution, and VGA text-mode dependencies are
explicitly out of scope.

## Minimum Firmware Requirements

To be considered a supported boot environment for Oxide, firmware must provide:

- **UEFI 2.x compliant implementation**  
  Minor revision differences are tolerated as long as required protocols are
  available and functional.

- **Graphics Output Protocol (GOP)**  
  Required for early framebuffer discovery and deterministic graphical output.

- **Boot Services memory map access**  
  The loader must be able to retrieve a complete and accurate UEFI memory map
  prior to exiting boot services.

- **64-bit execution environment**  
  The firmware must launch UEFI applications in 64-bit long mode.

Firmware-provided text console services are considered optional conveniences and
must not be relied upon after early bring-up.

## Validation Targets

To keep loader development portable and verifiable, UEFI support is validated
against the following environments:

- **OVMF (EDK II)**  
  Used as the primary reference implementation for development, debugging, and
  CI-style testing.

- **Real hardware with vendor UEFI firmware**  
  At least one class of contemporary x86_64 hardware is expected to be used to
  validate assumptions beyond emulation.

No reliance on vendor-specific extensions or undocumented firmware behavior is
permitted.

## Consequences

- The bootloader is significantly simpler and more robust.
- The project avoids real-mode and protected-mode transition complexity.
- Early development velocity is improved by leveraging standardized firmware
  services.
- Oxide will not run on systems lacking UEFI support.
- Firmware services must not leak beyond the boot boundary into the kernel.

These tradeoffs are considered acceptable and aligned with the project’s
objectives.

## Future Considerations

UEFI support does not imply long-term dependence on firmware services.

After boot:
- No UEFI APIs are available or relied upon.
- All hardware resources are owned and managed by the kernel.

If future platforms or architectures are supported, they will be evaluated
under similar criteria: modern boot environments, explicit boundaries, and
minimal legacy constraints.
