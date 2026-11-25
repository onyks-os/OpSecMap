# Harden the computer that opens your maps

An encrypted map file is safe at rest. While a map is open, though, the
program holds it, and briefly the password, in memory, and
[not every copy is cleared](https://github.com/onyks-os/OpSecMap/blob/main/docs/encrypted-format.md#limits).
The operating system can then write that memory to disk: to swap, to a
hibernation image, or to a crash dump. These steps reduce where such copies end
up. They are optional and need no change to OpSecMap.

None of them protects a map from malware or another person using your account
while the map is open: they can read it directly.

## 1. Encrypt the whole disk

This matters most, because it also covers swap, hibernation and crash dumps
written to that disk.

- **Linux:** LUKS, usually chosen when installing the system.
- **macOS:** FileVault.
- **Windows:** BitLocker or Device Encryption.

## 2. Check swap and hibernation

On Linux, `swapon --show` lists the swap in use. Swap on `zram` stays in memory;
a swap partition or file should be on the encrypted disk. Hibernation writes all
memory to disk, so use it only with an encrypted disk, or disable it.

## 3. Disable crash dumps

If the program crashes, the system can save its memory, including an open map,
to a file.

- In the shell where you start OpSecMap, run `ulimit -c 0` first.
- On Linux with systemd, `coredumpctl list` shows saved dumps. To stop saving
  them for every program, create `/etc/systemd/coredump.conf.d/disable.conf`
  with:

  ```ini
  [Coredump]
  Storage=none
  ProcessSizeMax=0
  ```

## 4. Use a memory allocator that clears freed memory (Linux)

OpSecMap, like most Rust programs, gets memory from the system allocator. When
a buffer is freed, for instance a copy that `serde_json` made while reading a
map, its contents usually stay in memory until the space is reused.

[hardened_malloc](https://github.com/GrapheneOS/hardened_malloc), from
GrapheneOS, can replace the system allocator without rebuilding OpSecMap. By
default it overwrites small allocations with zeros when they are freed, and
returns large ones to the kernel, which clears them before giving them to
another program. It also adds guard pages and randomization against memory
corruption.

It does **not** clear data the program is still using, the stack, the CPU
registers or the terminal's buffers, and it does not keep memory out of swap.
It costs some memory and speed.

Build it from source and start OpSecMap with it:

```sh
git clone https://github.com/GrapheneOS/hardened_malloc.git
cd hardened_malloc
make
LD_PRELOAD="$PWD/out/libhardened_malloc.so" opsecmap
```

On 2026-11-26 the project ran its whole test suite, the demo and the terminal
password checks under hardened_malloc (commit `bf37d54`, default
configuration) on Fedora, with no difference in behavior. If OpSecMap behaves
differently with it on your system, please
[open an issue](https://github.com/onyks-os/OpSecMap/issues).

## 5. Keep the password out of the environment

`OPSECMAP_PASSWORD` is visible to other programs running as you and can end up
in shell history. Use it only with fictional maps; let OpSecMap ask for a real
password.
