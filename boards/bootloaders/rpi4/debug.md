```zsh

❯ terminal-s.exe
--- COM3 is connected. Press Ctrl+] to quit ---

[    0.424055] EMMC: reset card.
[    0.424153] control1: 16143
[    0.426768] Divisor = 63, Freq Set = 396825
[    0.834204] CSD Contents : 00 40 0e 00 32 5b 59 00 00ed c8 7f 80 0a 40 40
[    0.838032] cemmc_structure=1, spec_vers=0, taac=0x0E, nsac=0x00, tran_speed=0x32,ccc=0x05B5, read_bl_len=0x09, read_bl_partial=0b, write_blk_misalign=0b,read_blk_misalign=0b, dsr_imp=0b, sector_size =0x7F, erase_blk_en=1b
[    0.857663] CSD 2.0: ver2_c_size = 0xEFFC, card capacity: 31914459136 bytes or 31.91GiB
[    0.865569] wp_grp_size=0x0000000b, wp_grp_enable=0b, default_ecc=00b, r2w_factor=010b, write_bl_len=0x09, write_bl_partial=0b, file_format_grp=0, copy=1b, perm_write_protect=0b, tmp_write_protect=0b, file_format=0b ecc=00b
[    0.885292] control1: 271
[    0.887809] Divisor = 1, Freq Set = 25000000
[    0.894327] EMMC: Bus width set to 4
[    0.895462] EMMC: SD Card Type 2 HC, 30436Mb, mfr_id: 3, 'SD:ACLCD', r8.0, mfr_date: 1/2017, serial: 0xbbce119c, RCA: 0xaaaa
[    0.906573] EMMC2 driver initialized...

[    0.910397] rpi4 version 0.1.0
[    0.913349] Booting on: Raspberry Pi 4
[    0.916999] Current privilege level: EL1
[    0.920821] Exception handling state:
[    0.924383]       Debug:  Masked
[    0.927510]       SError: Masked
[    0.930638]       IRQ:    Masked
[    0.933765]       FIQ:    Masked
[    0.936893] Architectural timer resolution: 18 ns
[    0.941497] Drivers loaded:
[    0.944190]       1. BCM GPIO
[    0.947057]       2. BCM PL011 UART
[    0.950445] Chars written: 1482
[W   0.953488] wait duration smaller than architecturally supported, skipping
[    0.960262] create new emmc-fat controller...
[    0.969024]  Listing root directory:

[    0.970136]          Found: DirEntry { name: ShortFileName("BOOT"), mtime: Timestamp(2022-01-27 01:53:38), ctime: Timestamp(1980-01-01 00:00:00), attributes: FV, cluster: Cluster(0), size: 0, entry_block: BlockIdx(10240), entry_offset: 0 }
[    0.990236]          Found: DirEntry { name: ShortFileName("SYSTEM~1"), mtime: Timestamp(2022-01-26 15:05:42), ctime: Timestamp(2022-01-26 15:05:40), attributes: DHS, cluster: Cluster(3), size: 0, entry_block: BlockIdx(10240), entry_offset: 96 }
[    1.011260]          Found: DirEntry { name: ShortFileName("VMLINUZ"), mtime: Timestamp(2022-01-08 18:37:12), ctime: Timestamp(2022-01-27 01:54:52), attributes: FA, cluster: Cluster(6), size: 29272576, entry_block: BlockIdx(10240), entry_offset: 128 }
[    1.032803]          Found: DirEntry { name: ShortFileName("INITRA~1"), mtime: Timestamp(2022-01-14 16:21:24), ctime: Timestamp(2022-01-27 01:55:34), attributes: FA, cluster: Cluster(7153), size: 32901194, entry_block: BlockIdx(10240), entry_offset: 192 }
[    1.054697]          Found: DirEntry { name: ShortFileName("FIXUP4.DAT"), mtime: Timestamp(2021-04-30 14:01:38), ctime: Timestamp(2022-01-27 10:23:18), attributes: FA, cluster: Cluster(15193), size: 5446, entry_block: BlockIdx(10240), entry_offset: 320 }
[    1.076501]          Found: DirEntry { name: ShortFileName("START4.ELF"), mtime: Timestamp(2021-04-30 14:01:38), ctime: Timestamp(2022-01-27 10:23:18), attributes: FA, cluster: Cluster(15195), size: 2228768, entry_block: BlockIdx(10240), entry_offset: 352 }
[    1.098568]          Found: DirEntry { name: ShortFileName("CONFIG.TXT"), mtime: Timestamp(2022-01-04 11:46:38), ctime: Timestamp(2022-01-27 10:23:18), attributes: FA, cluster: Cluster(15740), size: 1846, entry_block: BlockIdx(10240), entry_offset: 384 }
[    1.122874]          Found: DirEntry { name: ShortFileName("KERNEL8.IMG"), mtime: Timestamp(2022-01-28 15:35:02), ctime: Timestamp(2022-01-28 15:11:30), attributes: FA, cluster: Cluster(15741), size: 86136, entry_block: BlockIdx(10241), entry_offset: 288 }
[    1.142355]          Found: DirEntry { name: ShortFileName("BCM271~1.DTB"), mtime: Timestamp(2022-01-28 20:57:38), ctime: Timestamp(2022-01-27 10:22:42), attributes: FA, cluster: Cluster(15186), size: 25934, entry_block: BlockIdx(10241), entry_offset: 384 }
[    1.164410] Get handle to `dtb` file in root_dir...
[    1.172192]          load `dtb` into RAM...
[    1.181161]          loaded dtb: 25934 bytes, starting at addr: 0x200000
[    1.184294] Get handle to `kernel` file in root_dir...
[    1.192057]          load `kernel` into RAM...
[    8.649090]          loaded kernel: 29272576 bytes, starting at addr: 0x400000
[    8.652745] Get handle to `initramfs` file in root_dir...
[    8.660733]          load `initramfs` into RAM...
[   17.015866]          loaded initramfs: 32901194 bytes, starting at addr: 0x5890000

[   17.019954] ***************************************** Starting kernel ********************************************

[    0.000000] Booting Linux on physical CPU 0x0000000000 [0x410fd083]
[    0.000000] Linux version 5.15.0-trunk-arm64 (debian-kernel@lists.debian.org) (gcc-10 (Apertis 10.2.1-6+apertis3bv2022preb4) 10.2.1 20210110, GNU ld (GNU Binutils for Apertis) 2.35.2) #1 SMP Debian 5.15.1-1~exp1+apertis1 (2021-11-11)
[    0.000000] Machine model: Raspberry Pi 4 Model B
[    0.000000] efi: UEFI not found.
[    0.000000] Reserved memory: bypass linux,cma node, using cmdline CMA params instead
[    0.000000] OF: reserved mem: node linux,cma compatible matching fail
[    0.000000] NUMA: No NUMA configuration found
[    0.000000] NUMA: Faking a node at [mem 0x0000000000000000-0x00000000fbffffff]
[    0.000000] NUMA: NODE_DATA [mem 0xfb814b00-0xfb816fff]
[    0.000000] Zone ranges:
[    0.000000]   DMA      [mem 0x0000000000000000-0x000000003fffffff]
[    0.000000]   DMA32    [mem 0x0000000040000000-0x00000000fbffffff]
[    0.000000]   Normal   empty
[    0.000000] Movable zone start for each node
[    0.000000] Early memory node ranges
[    0.000000]   node   0: [mem 0x0000000000000000-0x000000003b3fffff]
[    0.000000]   node   0: [mem 0x0000000040000000-0x00000000fbffffff]
[    0.000000] Initmem setup node 0 [mem 0x0000000000000000-0x00000000fbffffff]
[    0.000000] cma: Reserved 128 MiB at 0x0000000033400000
[    0.000000] percpu: Embedded 29 pages/cpu s81752 r8192 d28840 u118784
[    0.000000] Detected PIPT I-cache on CPU0
[    0.000000] CPU features: detected: Spectre-v2
[    0.000000] CPU features: detected: Spectre-v3a
[    0.000000] CPU features: detected: Spectre-v4
[    0.000000] CPU features: detected: ARM errata 1165522, 1319367, or 1530923
[    0.000000] Built 1 zonelists, mobility grouping on.  Total pages: 996912
[    0.000000] Policy zone: DMA32
[    0.000000] Kernel command line: root=UUID=f2fa8d24-c392-4176-ab1c-367d60b66c6a rootwait ro plymouth.ignore-serial-consoles fsck.mode=auto fsck.repair=yes cma=128M
[    0.000000] Dentry cache hash table entries: 524288 (order: 10, 4194304 bytes, linear)
[    0.000000] Inode-cache hash table entries: 262144 (order: 9, 2097152 bytes, linear)
[    0.000000] mem auto-init: stack:off, heap alloc:on, heap free:off
[    0.000000] software IO TLB: mapped [mem 0x000000002f400000-0x0000000033400000] (64MB)
[    0.000000] Memory: 843888K/4050944K available (12352K kernel code, 2538K rwdata, 7808K rodata, 5760K init, 622K bss, 208032K reserved, 131072K cma-reserved)
[    0.000000] random: get_random_u64 called from __kmem_cache_create+0x34/0x5cc with crng_init=0
[    0.000000] SLUB: HWalign=64, Order=0-3, MinObjects=0, CPUs=4, Nodes=1
[    0.000000] ftrace: allocating 40621 entries in 159 pages
[    0.000000] ftrace: allocated 159 pages with 6 groups
[    0.000000] trace event string verifier disabled
[    0.000000] rcu: Hierarchical RCU implementation.
[    0.000000] rcu:     RCU restricting CPUs from NR_CPUS=256 to nr_cpu_ids=4.
[    0.000000]  Rude variant of Tasks RCU enabled.
[    0.000000]  Tracing variant of Tasks RCU enabled.
[    0.000000] rcu: RCU calculated value of scheduler-enlistment delay is 25 jiffies.
[    0.000000] rcu: Adjusting geometry for rcu_fanout_leaf=16, nr_cpu_ids=4
[    0.000000] NR_IRQS: 64, nr_irqs: 64, preallocated irqs: 0
[    0.000000] Root IRQ handler: gic_handle_irq
[    0.000000] arch_timer: cp15 timer(s) running at 54.00MHz (virt).
[    0.000000] clocksource: arch_sys_counter: mask: 0xffffffffffffff max_cycles: 0xc743ce346, max_idle_ns: 440795203123 ns
[    0.000001] sched_clock: 56 bits at 54MHz, resolution 18ns, wraps every 4398046511102ns
[    0.000221] Console: colour dummy device 80x25
[    0.000590] printk: console [tty0] enabled
[    0.000690] Calibrating delay loop (skipped), value calculated using timer frequency.. 108.00 BogoMIPS (lpj=216000)
[    0.000720] pid_max: default: 32768 minimum: 301
[    0.000868] LSM: Security Framework initializing
[    0.000908] Yama: disabled by default; enable with sysctl kernel.yama.*
[    0.001060] AppArmor: AppArmor initialized
[    0.001081] TOMOYO Linux initialized
[    0.001223] Mount-cache hash table entries: 8192 (order: 4, 65536 bytes, linear)
[    0.001302] Mountpoint-cache hash table entries: 8192 (order: 4, 65536 bytes, linear)
[    0.003802] rcu: Hierarchical SRCU implementation.
[    0.006011] EFI services will not be available.
[    0.006572] smp: Bringing up secondary CPUs ...
[    0.007254] Detected PIPT I-cache on CPU1
[    0.007329] CPU1: Booted secondary processor 0x0000000001 [0x410fd083]
[    0.008175] Detected PIPT I-cache on CPU2
[    0.008222] CPU2: Booted secondary processor 0x0000000002 [0x410fd083]
[    0.009034] Detected PIPT I-cache on CPU3
[    0.009080] CPU3: Booted secondary processor 0x0000000003 [0x410fd083]
[    0.009190] smp: Brought up 1 node, 4 CPUs
[    0.009256] SMP: Total of 4 processors activated.
[    0.009270] CPU features: detected: 32-bit EL0 Support
[    0.009281] CPU features: detected: 32-bit EL1 Support
[    0.009295] CPU features: detected: CRC32 instructions
[    0.027632] ------------[ cut here ]------------
[    0.027668] CPU: CPUs started in inconsistent modes
[    0.027682] WARNING: CPU: 0 PID: 1 at arch/arm64/kernel/smp.c:426 smp_cpus_done+0x78/0xc4
[    0.027735] Modules linked in:
[    0.027753] CPU: 0 PID: 1 Comm: swapper/0 Not tainted 5.15.0-trunk-arm64 #1  Debian 5.15.1-1~exp1+apertis1
[    0.027778] Hardware name: Raspberry Pi 4 Model B (DT)
[    0.027792] pstate: 60000005 (nZCv daif -PAN -UAO -TCO -DIT -SSBS BTYPE=--)
[    0.027813] pc : smp_cpus_done+0x78/0xc4
[    0.027833] lr : smp_cpus_done+0x78/0xc4
[    0.027852] sp : ffff800011cabe00
[    0.027863] x29: ffff800011cabe00 x28: 0000000000000000 x27: 0000000000000000
[    0.027892] x26: 0000000000000000 x25: 0000000000000000 x24: 0000000000000000
[    0.027918] x23: 0000000000000000 x22: ffff800011bf1000 x21: 0000000000000000
[    0.027942] x20: ffff800011527e40 x19: ffff800011a20000 x18: 0000000000000001
[    0.027967] x17: 00000000ecb96905 x16: 000000004750aae2 x15: 0720072007200720
[    0.027992] x14: 0720072d072d072d x13: 7365646f6d20746e x12: 65747369736e6f63
[    0.028017] x11: ffff800011a00288 x10: ffff8000119a9588 x9 : ffff80001011cc1c
[    0.028042] x8 : ffff8000119a7b68 x7 : ffff8000119ffb68 x6 : fffffffffffe1418
[    0.028066] x5 : 0000000000001a20 x4 : 000000000000aff5 x3 : 0000000000000000
[    0.028090] x2 : 0000000000000000 x1 : 0000000000000000 x0 : ffff0000401bbd00
[    0.028115] Call trace:
[    0.028127]  smp_cpus_done+0x78/0xc4
[    0.028148]  smp_init+0x88/0x98
[    0.028170]  kernel_init_freeable+0x194/0x328
[    0.028190]  kernel_init+0x30/0x140
[    0.028207]  ret_from_fork+0x10/0x20
[    0.028233] ---[ end trace 334c9d2382cabd46 ]---
[    0.028338] alternatives: patching kernel code
[    0.118418] node 0 deferred pages initialised in 88ms
[    0.120019] devtmpfs: initialized
[    0.128373] Registered cp15_barrier emulation handler
[    0.128424] Registered setend emulation handler
[    0.128443] KASLR disabled due to lack of seed
[    0.128751] clocksource: jiffies: mask: 0xffffffff max_cycles: 0xffffffff, max_idle_ns: 7645041785100000 ns
[    0.128847] futex hash table entries: 1024 (order: 4, 65536 bytes, linear)
[    0.133477] pinctrl core: initialized pinctrl subsystem
[    0.134261] DMI not present or invalid.
[    0.134917] NET: Registered PF_NETLINK/PF_ROUTE protocol family
[    0.139976] DMA: preallocated 512 KiB GFP_KERNEL pool for atomic allocations
[    0.140859] DMA: preallocated 512 KiB GFP_KERNEL|GFP_DMA pool for atomic allocations
[    0.141951] DMA: preallocated 512 KiB GFP_KERNEL|GFP_DMA32 pool for atomic allocations
[    0.142089] audit: initializing netlink subsys (disabled)
[    0.142413] audit: type=2000 audit(0.140:1): state=initialized audit_enabled=0 res=1
[    0.143601] thermal_sys: Registered thermal governor 'fair_share'
[    0.143610] thermal_sys: Registered thermal governor 'bang_bang'
[    0.143629] thermal_sys: Registered thermal governor 'step_wise'
[    0.143642] thermal_sys: Registered thermal governor 'user_space'
[    0.143655] thermal_sys: Registered thermal governor 'power_allocator'
[    0.143886] cpuidle: using governor ladder
[    0.143932] cpuidle: using governor menu
[    0.144091] hw-breakpoint: found 6 breakpoint and 4 watchpoint registers.
[    0.144224] ASID allocator initialised with 65536 entries
[    0.144692] Serial: AMBA PL011 UART driver
[    0.171419] HugeTLB registered 1.00 GiB page size, pre-allocated 0 pages
[    0.171462] HugeTLB registered 32.0 MiB page size, pre-allocated 0 pages
[    0.171479] HugeTLB registered 2.00 MiB page size, pre-allocated 0 pages
[    0.171495] HugeTLB registered 64.0 KiB page size, pre-allocated 0 pages
[    0.830334] ACPI: Interpreter disabled.
[    0.830977] iommu: Default domain type: Translated
[    0.830997] iommu: DMA domain TLB invalidation policy: strict mode
[    0.831325] vgaarb: loaded
[    0.831653] EDAC MC: Ver: 3.0.0
[    0.833448] NetLabel: Initializing
[    0.833474] NetLabel:  domain hash size = 128
[    0.833489] NetLabel:  protocols = UNLABELED CIPSOv4 CALIPSO
[    0.833609] NetLabel:  unlabeled traffic allowed by default
[    0.834051] clocksource: Switched to clocksource arch_sys_counter
[    0.879261] VFS: Disk quotas dquot_6.6.0
[    0.879369] VFS: Dquot-cache hash table entries: 512 (order 0, 4096 bytes)
[    0.880243] AppArmor: AppArmor Filesystem Enabled
[    0.880473] pnp: PnP ACPI: disabled
[    0.889395] NET: Registered PF_INET protocol family
[    0.889951] IP idents hash table entries: 65536 (order: 7, 524288 bytes, linear)
[    0.893033] tcp_listen_portaddr_hash hash table entries: 2048 (order: 3, 32768 bytes, linear)
[    0.893349] TCP established hash table entries: 32768 (order: 6, 262144 bytes, linear)
[    0.894080] TCP bind hash table entries: 32768 (order: 7, 524288 bytes, linear)
[    0.894392] TCP: Hash tables configured (established 32768 bind 32768)
[    0.894877] MPTCP token hash table entries: 4096 (order: 4, 98304 bytes, linear)
[    0.895087] UDP hash table entries: 2048 (order: 4, 65536 bytes, linear)
[    0.895210] UDP-Lite hash table entries: 2048 (order: 4, 65536 bytes, linear)
[    0.895900] NET: Registered PF_UNIX/PF_LOCAL protocol family
[    0.895947] NET: Registered PF_XDP protocol family
[    0.895969] PCI: CLS 0 bytes, default 64
[    0.896749] Trying to unpack rootfs image as initramfs...
[    0.903327] hw perfevents: enabled with armv8_cortex_a72 PMU driver, 7 counters available
[    0.903535] kvm [1]: HYP mode not available
[    0.904995] Initialise system trusted keyrings
[    0.905051] Key type blacklist registered
[    0.905310] workingset: timestamp_bits=42 max_order=20 bucket_order=0
[    0.911830] zbud: loaded
[    0.912728] integrity: Platform Keyring initialized
[    0.912755] Key type asymmetric registered
[    0.912769] Asymmetric key parser 'x509' registered
[    0.912891] Block layer SCSI generic (bsg) driver version 0.4 loaded (major 248)
[    0.913184] io scheduler mq-deadline registered
[    0.921554] shpchp: Standard Hot Plug PCI Controller Driver version: 0.4
[    0.922514] brcm-pcie fd500000.pcie: host bridge /scb/pcie@7d500000 ranges:
[    0.922553] brcm-pcie fd500000.pcie:   No bus range found for /scb/pcie@7d500000, using [bus 00-ff]
[    0.922602] brcm-pcie fd500000.pcie:      MEM 0x0600000000..0x0603ffffff -> 0x00f8000000
[    0.922646] brcm-pcie fd500000.pcie:   IB MEM 0x0000000000..0x00bfffffff -> 0x0000000000
[    0.988144] brcm-pcie fd500000.pcie: link up, 5.0 GT/s PCIe x1 (SSC)
[    0.988480] brcm-pcie fd500000.pcie: PCI host bridge to bus 0000:00
[    0.988503] pci_bus 0000:00: root bus resource [bus 00-ff]
[    0.988524] pci_bus 0000:00: root bus resource [mem 0x600000000-0x603ffffff] (bus address [0xf8000000-0xfbffffff])
[    0.988584] pci 0000:00:00.0: [14e4:2711] type 01 class 0x060400
[    0.988683] pci 0000:00:00.0: PME# supported from D0 D3hot
[    0.991238] pci 0000:01:00.0: [1106:3483] type 00 class 0x0c0330
[    0.991328] pci 0000:01:00.0: reg 0x10: [mem 0x00000000-0x00000fff 64bit]
[    0.991523] pci 0000:01:00.0: PME# supported from D0 D3hot
[    1.004127] pci 0000:00:00.0: BAR 14: assigned [mem 0x600000000-0x6000fffff]
[    1.004173] pci 0000:01:00.0: BAR 0: assigned [mem 0x600000000-0x600000fff 64bit]
[    1.004205] pci 0000:00:00.0: PCI bridge to [bus 01]
[    1.004223] pci 0000:00:00.0:   bridge window [mem 0x600000000-0x6000fffff]
[    1.004481] pcieport 0000:00:00.0: enabling device (0000 -> 0002)
[    1.004674] pcieport 0000:00:00.0: PME: Signaling with IRQ 50
[    1.005079] pcieport 0000:00:00.0: AER: enabled with IRQ 50
[    1.016339] Serial: 8250/16550 driver, 4 ports, IRQ sharing enabled
[    1.018488] fe215040.serial: ttyS1 at MMIO 0xfe215040 (irq = 26, base_baud = 24999999) is a 16550
[    2.165917] printk: console [ttyS1] enabled
[    2.171345] Serial: AMBA driver
[    2.174622] SuperH (H)SCI(F) driver initialized
[    2.179807] msm_serial: driver initialized
[    2.185061] cacheinfo: Unable to detect cache hierarchy for CPU 0
[    2.192163] bcm2835-power bcm2835-power: Broadcom BCM2835 power domains driver
[    2.200546] mousedev: PS/2 mouse device common for all mice
[    2.207464] brcmstb-i2c fef04500.i2c:  @97500hz registered in polling mode
[    2.214797] brcmstb-i2c fef09500.i2c:  @97500hz registered in polling mode
[    2.223471] ledtrig-cpu: registered to indicate activity on CPUs
[    2.230536] bcm2835-mbox fe00b880.mailbox: mailbox enabled
[    2.238365] NET: Registered PF_INET6 protocol family
[    3.336190] Freeing initrd memory: 32128K
[    3.378472] Segment Routing with IPv6
[    3.382332] In-situ OAM (IOAM) with IPv6
[    3.386454] mip6: Mobile IPv6
[    3.389476] NET: Registered PF_PACKET protocol family
[    3.394817] mpls_gso: MPLS GSO support
[    3.399436] registered taskstats version 1
[    3.403645] Loading compiled-in X.509 certificates
[    3.558820] Loaded X.509 cert 'Debian Secure Boot CA: 6ccece7e4c6c0d1f6149f3dd27dfcc5cbb419ea1'
[    3.567764] Loaded X.509 cert 'Debian Secure Boot Signer 2021 - linux: 4b6ef5abca669825178e052c84667ccbc0531f8c'
[    3.579102] zswap: loaded using pool lzo/zbud
[    3.584451] Key type ._fscrypt registered
[    3.588563] Key type .fscrypt registered
[    3.592566] Key type fscrypt-provisioning registered
[    3.615184] Key type encrypted registered
[    3.619310] AppArmor: AppArmor sha1 policy hashing enabled
[    3.624938] ima: No TPM chip found, activating TPM-bypass!
[    3.630552] ima: Allocated hash algorithm: sha256
[    3.635396] ima: No architecture policies found
[    3.640092] evm: Initialising EVM extended attributes:
[    3.645329] evm: security.selinux
[    3.648708] evm: security.SMACK64 (disabled)
[    3.653056] evm: security.SMACK64EXEC (disabled)
[    3.657756] evm: security.SMACK64TRANSMUTE (disabled)
[    3.662899] evm: security.SMACK64MMAP (disabled)
[    3.667600] evm: security.apparmor
[    3.671063] evm: security.ima
[    3.674085] evm: security.capability
[    3.677718] evm: HMAC attrs: 0x1
[    3.686537] fe201000.serial: ttyAMA0 at MMIO 0xfe201000 (irq = 24, base_baud = 0) is a PL011 rev2
[    3.695930] serial serial0: tty port ttyAMA0 registered
[    3.702422] raspberrypi-firmware soc:firmware: Attached to firmware from 2021-04-30T13:45:52
[    3.885790] Freeing unused kernel memory: 5760K
[    3.947735] Checked W+X mappings: passed, no W+X pages found
[    3.953550] Run /init as init process
Loading, please wait...
Starting version 247.3-6+apertis1bv2022dev3b2
[    4.325330] phy_generic: module verification failed: signature and/or required key missing - tainting kernel
[    4.422021] sdhci: Secure Digital Host Controller Interface driver
[    4.428428] sdhci: Copyright(c) Pierre Ossman
[    4.433514] usb_phy_generic phy: supply vcc not found, using dummy regulator
[    4.444676] sdhci-pltfm: SDHCI platform and OF driver helper
[    4.445309] libphy: Fixed MDIO Bus: probed
[    4.470322] usbcore: registered new interface driver usbfs
[    4.476110] usbcore: registered new interface driver hub
[    4.482188] usbcore: registered new device driver usb
[    4.506992] bcmgenet fd580000.ethernet: GENET 5.0 EPHY: 0x0000
[    4.513126] bcmgenet fd580000.ethernet: using random Ethernet MAC
[    4.534796] sdhci-iproc fe300000.mmc: allocated mmc-pwrseq
[    4.552497] mmc0: SDHCI controller on fe340000.mmc [fe340000.mmc] using ADMA
[    4.558191] libphy: bcmgenet MII bus: probed
[    4.589182] xhci_hcd 0000:01:00.0: xHCI Host Controller
[    4.594650] xhci_hcd 0000:01:00.0: new USB bus registered, assigned bus number 1
[    4.602859] xhci_hcd 0000:01:00.0: hcc params 0x002841eb hci version 0x100 quirks 0x0000040000000890
[    4.613047] usb usb1: New USB device found, idVendor=1d6b, idProduct=0002, bcdDevice= 5.15
[    4.613559] dwc2 fe980000.usb: supply vusb_d not found, using dummy regulator
[    4.621530] usb usb1: New USB device strings: Mfr=3, Product=2, SerialNumber=1
[    4.621547] usb usb1: Product: xHCI Host Controller
[    4.629039] dwc2 fe980000.usb: supply vusb_a not found, using dummy regulator
[    4.636266] usb usb1: Manufacturer: Linux 5.15.0-trunk-arm64 xhci-hcd
[    4.636277] usb usb1: SerialNumber: 0000:01:00.0
[    4.650187] unimac-mdio unimac-mdio.-19: Broadcom UniMAC MDIO bus
[    4.651555] mmc1: SDHCI controller on fe300000.mmc [fe300000.mmc] using PIO
[    4.655955] hub 1-0:1.0: USB hub found
[    4.677205] hub 1-0:1.0: 1 port detected
[    4.682353] xhci_hcd 0000:01:00.0: xHCI Host Controller
[    4.687828] xhci_hcd 0000:01:00.0: new USB bus registered, assigned bus number 2
[    4.695443] xhci_hcd 0000:01:00.0: Host supports USB 3.0 SuperSpeed
[    4.702324] usb usb2: New USB device found, idVendor=1d6b, idProduct=0003, bcdDevice= 5.15
[    4.710799] usb usb2: New USB device strings: Mfr=3, Product=2, SerialNumber=1
[    4.718202] usb usb2: Product: xHCI Host Controller
[    4.723212] usb usb2: Manufacturer: Linux 5.15.0-trunk-arm64 xhci-hcd
[    4.729811] usb usb2: SerialNumber: 0000:01:00.0
[    4.735332] hub 2-0:1.0: USB hub found
[    4.739278] hub 2-0:1.0: 4 ports detected
[    4.743842] dwc2 fe980000.usb: EPs: 8, dedicated fifos, 4080 entries in SPRAM
[    4.775312] mmc0: new ultra high speed DDR50 SDHC card at address aaaa
Begin: Loading essential[    4.783480] mmcblk0: mmc0:aaaa ACLCD 29.7 GiB
 drivers[    4.789274] random: fast init done
 ... [    4.798439]  mmcblk0: p1 p2
[    4.835304] mmc1: new high speed SDIO card at address 0001
done.
Begin: Running /scripts/init-premount ... done.
Begin: Mounting root file system ... Begin: Running /scripts/local-top ... done.
Begin: Running /scripts/local-premount ... done.
[    4.946108] usb 1-1: new high-speed USB device number 2 using xhci_hcd
Begin: Will now check root file system ... fsck from util-linux 2.36.1
[/sbin/fsck.ext4 (1) -- /dev/mmcblk0p2] fsck.ext4 -y -C0 /dev/mmcblk0p2
e2fsck 1.46.2 (28-Feb-2021)
system: clean, 13170/183264 files, 206501/732416 blocks
done.
[    5.108789] usb 1-1: New USB device found, idVendor=2109, idProduct=3431, bcdDevice= 4.21
[    5.117311] usb 1-1: New USB device strings: Mfr=0, Product=1, SerialNumber=0
[    5.124659] usb 1-1: Product: USB2.0 Hub
[    5.130952] hub 1-1:1.0: USB hub found
[    5.135111] hub 1-1:1.0: 4 ports detected
[    5.155928] EXT4-fs (mmcblk0p2): mounted filesystem with ordered data mode. Opts: (null). Quota mode: none.
done.
Begin: Running /scripts/local-bottom ... done.
Begin: Running /scripts/init-bottom ... done.
[    5.408627] Not activating Mandatory Access Control as /sbin/tomoyo-init does not exist.
[    5.731582] systemd[1]: System time before build time, advancing clock.
[    5.786032] systemd[1]: Inserted module 'autofs4'
[    5.853131] systemd[1]: systemd 247.3-6+apertis1bv2022dev3b2 running in system mode. (+PAM +AUDIT +SELINUX +IMA +APPARMOR +SMACK +SYSVINIT +UTMP +LIBCRYPTSETUP +GCRYPT -GNUTLS +ACL +XZ +LZ4 +ZSTD +SECCOMP +BLKID -ELFUTILS +KMOD +IDN2 -IDN +PCRE2 default-hierarchy=unified)
[    5.878330] systemd[1]: Detected architecture arm64.

Welcome to Apertis v2022pre!

[    5.905150] systemd[1]: Set hostname to <apertis>.
[    5.914166] random: systemd: uninitialized urandom read (16 bytes read)
[    5.920971] systemd[1]: Initializing machine ID from random generator.
[    5.927842] systemd[1]: Installed transient /etc/machine-id file.
[    6.441169] systemd[1]: Queued start job for default target Graphical Interface.
[    6.449836] random: systemd: uninitialized urandom read (16 bytes read)
[    6.460138] systemd[1]: Created slice system-getty.slice.
[  OK  ] Created slice system-getty.slice.
[    6.482279] random: systemd: uninitialized urandom read (16 bytes read)
[    6.490571] systemd[1]: Created slice system-modprobe.slice.
[  OK  ] Created slice system-modprobe.slice.
[    6.516022] systemd[1]: Created slice system-serial\x2dgetty.slice.
[  OK  ] Created slice system-serial\x2dgetty.slice.
[    6.539558] systemd[1]: Created slice system-systemd\x2dfsck.slice.
[  OK  ] Created slice system-systemd\x2dfsck.slice.
[    6.563412] systemd[1]: Created slice User and Session Slice.
[  OK  ] Created slice User and Session Slice.
[    6.586559] systemd[1]: Started Dispatch Password Requests to Console Directory Watch.
[  OK  ] Started Dispatch Password …ts to Console Directory Watch.
[    6.614461] systemd[1]: Started Forward Password Requests to Wall Directory Watch.
[  OK  ] Started Forward Password R…uests to Wall Directory Watch.
[    6.642956] systemd[1]: Set up automount Arbitrary Executable File Formats File System Automount Point.
[  OK  ] Set up automount Arbitrary…s File System Automount Point.
[    6.670366] systemd[1]: Reached target Local Encrypted Volumes.
[  OK  ] Reached target Local Encrypted Volumes.
[    6.694336] systemd[1]: Reached target Paths.
[  OK  ] Reached target Paths.
[    6.714224] systemd[1]: Reached target Remote File Systems.
[  OK  ] Reached target Remote File Systems.
[    6.734211] systemd[1]: Reached target Slices.
[  OK  ] Reached target Slices.
[    6.754259] systemd[1]: Reached target Swap.
[  OK  ] Reached target Swap.
[    6.774845] systemd[1]: Listening on fsck to fsckd communication Socket.
[  OK  ] Listening on fsck to fsckd communication Socket.
[    6.798501] systemd[1]: Listening on initctl Compatibility Named Pipe.
[  OK  ] Listening on initctl Compatibility Named Pipe.
[    6.823032] systemd[1]: Listening on Journal Audit Socket.
[  OK  ] Listening on Journal Audit Socket.
[    6.846873] systemd[1]: Listening on Journal Socket (/dev/log).
[  OK  ] Listening on Journal Socket (/dev/log).
[    6.870849] systemd[1]: Listening on Journal Socket.
[  OK  ] Listening on Journal Socket.
[    6.891073] systemd[1]: Listening on udev Control Socket.
[  OK  ] Listening on udev Control Socket.
[    6.914699] systemd[1]: Listening on udev Kernel Socket.
[  OK  ] Listening on udev Kernel Socket.
[    6.938549] systemd[1]: Mounting Huge Pages File System...
         Mounting Huge Pages File System...
[    6.963200] systemd[1]: Mounting POSIX Message Queue File System...
         Mounting POSIX Message Queue File System...
[    6.990946] systemd[1]: Mounting Kernel Debug File System...
         Mounting Kernel Debug File System...
[    7.015081] systemd[1]: Mounting Kernel Trace File System...
         Mounting Kernel Trace File System...
[    7.045416] systemd[1]: Mounting Temporary Directory (/tmp)...
         Mounting Temporary Directory (/tmp)...
[    7.071106] systemd[1]: Starting Create list of static device nodes for the current kernel...
         Starting Create list of st…odes for the current kernel...
[    7.102795] systemd[1]: Starting Load Kernel Module configfs...
         Starting Load Kernel Module configfs...
[    7.130773] systemd[1]: Starting Load Kernel Module drm...
         Starting Load Kernel Module drm...
[    7.154681] systemd[1]: Starting Load Kernel Module fuse...
         Starting Load Kernel Module fuse...
[    7.178408] systemd[1]: Condition check resulted in Set Up Additional Binary Formats being skipped.
[    7.188030] systemd[1]: Condition check resulted in File System Check on Root Device being skipped.
[    7.190178] fuse: init (API version 7.34)
[    7.204975] systemd[1]: Starting Journal Service...
         Starting Journal Service...
[    7.235435] systemd[1]: Starting Load Kernel Modules...
         Starting Load Kernel Modules...
[    7.259253] systemd[1]: Starting Remount Root and Kernel File Systems...
         Starting Remount Root and Kernel File Systems...
[    7.287499] systemd[1]: Starting Coldplug All udev Devices...
         Starting Coldplug All udev Devices...
[    7.320724] systemd[1]: Mounted Huge Pages File System.
[  OK  ] Mounted Huge Pages File System.
[    7.343345] systemd[1]: Mounted POSIX Message Queue File System.
[  OK  ] Mounted POSIX Message Queue File System.
[    7.367244] systemd[1]: Mounted Kernel Debug File System.
[  OK  ] Mounted Kernel Debug File System.
[    7.390979] systemd[1]: Mounted Kernel Trace File System.
[  OK  ] Mounted Kernel Trace File System.
[    7.415075] systemd[1]: Mounted Temporary Directory (/tmp).
[  OK  ] Mounted Temporary Directory (/tmp).
[    7.440496] systemd[1]: Finished Create list of static device nodes for the current kernel.
[  OK  ] Finished Create list of st… nodes for the current kernel.
[    7.467159] systemd[1]: Started Journal Service.
[  OK  ] Started Journal Service.
[  OK  ] Finished Load Kernel Module configfs.
[  OK  ] Finished Load Kernel Module drm.
[  OK  ] Finished Load Kernel Module fuse.
[  OK  ] Finished Load Kernel Modules.
[FAILED] Failed to start Remount Root and Kernel File Systems.
See 'systemctl status systemd-remount-fs.service' for details.
         Mounting FUSE Control File System...
         Mounting Kernel Configuration File System...
         Starting Flush Journal to Persistent Storage...
         Starting Load/Save Random Seed...
[    7.698457] systemd-journald[229]: Received client request to flush runtime journal.
         Starting Apply Kernel Variables...
         Starting Create Static Device Nodes in /dev...
[  OK  ] Mounted FUSE Control File System.
[  OK  ] Mounted Kernel Configuration File System.
[  OK  ] Finished Flush Journal to Persistent Storage.
[  OK  ] Finished Coldplug All udev Devices.
[FAILED] Failed to start Load/Save Random Seed.
See 'systemctl status systemd-random-seed.service' for details.
[  OK  ] Finished Apply Kernel Variables.
[  OK  ] Finished Create Static Device Nodes in /dev.
[  OK  ] Reached target Local File Systems (Pre).
         Mounting Mount points for removable media...
         Mounting root mount point...
         Starting Rule-based Manage…for Device Events and Files...
[  OK  ] Mounted Mount points for removable media.
[FAILED] Failed to mount root mount point.
See 'systemctl status root.mount' for details.
[  OK  ] Started Rule-based Manager for Device Events and Files.
[    8.342802] vchiq: module is from the staging directory, the quality is unknown, you have been warned.
[    8.499899] iproc-rng200 fe104000.rng: hwrng registered
[    8.511037] bcm2835-wdt bcm2835-wdt: Broadcom BCM2835 watchdog timer
[  OK  ] Found device /dev/ttyS1.
[  OK  ] Reached target Hardware activated USB gadget.
[    8.698821] alg: No test for fips(ansi_cprng) (fips_ansi_cprng)
[  OK  ] Listening on Load/Save RF …itch Status /dev/rfkill Watch.
[    8.828522] snd_bcm2835: module is from the staging directory, the quality is unknown, you have been warned.
[    8.831747] cfg80211: Loading compiled-in X.509 certificates for regulatory database
[    8.839584] mc: Linux media interface: v0.10
[    8.842892] vc4-drm gpu: bound fe400000.hvs (ops vc4_hvs_ops [vc4])
[    8.843193] vc4_hdmi fef00700.hdmi: IRQ hpd-connected not found
[    8.843204] vc4_hdmi fef00700.hdmi: IRQ hpd-removed not found
[    8.843270] vc4-drm gpu: failed to bind fef00700.hdmi (ops vc4_hdmi_ops [vc4]): -22
[    8.843511] vc4-drm gpu: master bind failed: -22
[    8.843538] vc4-drm: probe of gpu failed with error -22
[    8.847664] cfg80211: Loaded X.509 cert 'benh@debian.org: 577e021cb980e0e820821ba7b54b4961b8b4fadf'
[    8.896912] cfg80211: Loaded X.509 cert 'romain.perier@gmail.com: 3abbc6ec146e09d1b6016ab9d6cf71dd233f0328'
[    8.907464] cfg80211: Loaded X.509 cert 'sforshee: 00b28ddf47aef9cea7'
[    8.915414] platform regulatory.0: firmware: failed to load regulatory.db (-2)
[    8.922858] firmware_class: See https://wiki.debian.org/Firmware for information about missing firmware
[    8.925438] cryptd: max_cpu_qlen set to 1000
[    8.929784] videodev: Linux video capture interface: v2.00
[    8.932483] platform regulatory.0: Direct firmware load for regulatory.db failed with error -2
[    8.951394] cfg80211: failed to load regulatory.db
[    8.963834] bcm2835_audio bcm2835_audio: card created with 8 channels
[    8.966688] brcmfmac: brcmf_fw_alloc_request: using brcm/brcmfmac43455-sdio for chip BCM4345/6
[    8.980102] usbcore: registered new interface driver brcmfmac
[    8.986300] brcmfmac mmc1:0001:1: firmware: failed to load brcm/brcmfmac43455-sdio.raspberrypi,4-model-b.bin (-2)
[    8.996822] brcmfmac mmc1:0001:1: Direct firmware load for brcm/brcmfmac43455-sdio.raspberrypi,4-model-b.bin failed with error -2
[    9.012788] bcm2835_mmal_vchiq: module is from the staging directory, the quality is unknown, you have been warned.
[    9.043498] bcm2835_v4l2: module is from the staging directory, the quality is unknown, you have been warned.
[    9.046630] brcmfmac mmc1:0001:1: firmware: direct-loading firmware brcm/brcmfmac43455-sdio.bin
[    9.065515] brcmfmac mmc1:0001:1: firmware: direct-loading firmware brcm/brcmfmac43455-sdio.raspberrypi,4-model-b.txt
[  OK  ] Reached target Sound Card.
[    9.118713] random: crng init done
[    9.122195] random: 7 urandom warning(s) missed due to ratelimiting
[    9.228841] brcmfmac: brcmf_fw_alloc_request: using brcm/brcmfmac43455-sdio for chip BCM4345/6
[    9.240687] brcmfmac mmc1:0001:1: firmware: direct-loading firmware brcm/brcmfmac43455-sdio.clm_blob
[    9.254147] brcmfmac: brcmf_c_preinit_dcmds: Firmware: BCM4345/6 wl0: Sep 18 2020 02:27:58 version 7.45.221 (3a6d3a0 CY) FWID 01-bbd9282b
         Starting Load/Save RF Kill Switch Status...
[FAILED] Failed to start Load/Save RF Kill Switch Status.
See 'systemctl status systemd-rfkill.service' for details.
         Starting Load/Save RF Kill Switch Status...
[    9.359594] Bluetooth: Core ver 2.22
[    9.363370] NET: Registered PF_BLUETOOTH protocol family
[    9.368800] Bluetooth: HCI device and connection manager initialized
[    9.375319] Bluetooth: HCI socket layer initialized
[    9.380346] Bluetooth: L2CAP socket layer initialized
[    9.385530] Bluetooth: SCO socket layer initialized
[FAILED] Failed to start Load/Save RF Kill Switch Status.
See 'systemctl status systemd-rfkill.service' for details.
[    9.426483] Bluetooth: HCI UART driver ver 2.3
[    9.431062] Bluetooth: HCI UART protocol H4 registered
[    9.436523] Bluetooth: HCI UART protocol LL registered
[    9.441811] Bluetooth: HCI UART protocol ATH3K registered
[    9.447407] Bluetooth: HCI UART protocol Three-wire (H5) registered
        [    9.454027] Bluetooth: HCI UART protocol Intel registered
 Startin[    9.460236] Bluetooth: HCI UART protocol Broadcom registered
g     9.466545] Bluetooth: HCI UART protocol QCA registered
39mLoad/[    9.472468] Bluetooth: HCI UART protocol AG6XX registered
Save RF [    9.478688] Bluetooth: HCI UART protocol Marvell registered
Kill Swi[    9.483881] hci_uart_bcm serial0-0: supply vbat not found, using dummy regulator
tch Stat[    9.493436] hci_uart_bcm serial0-0: supply vddio not found, using dummy regulator
us...
[FAILED] Failed to start Load/Save RF Kill Switch Status.
See 'systemctl status systemd-rfkill.service' for details.
         Starting Load/Save RF Kill Switch Status...
[FAILED] Failed to start Load/Save RF Kill Switch Status.
See 'systemctl status systemd-rfkill.service' for details.
         Starting Load/Save RF Kill Switch Status...
[    9.630219] uart-pl011 fe201000.serial: no DMA platform data
[FAILED] Failed to start Load/Save RF Kill Switch Status.
See 'systemctl status systemd-rfkill.service' for details.
[FAILED] Failed to start Load/Save RF Kill Switch Status.
See 'systemctl status systemd-rfkill.service' for details.
[  OK  ] Reached target Bluetooth.
[   11.774069] Bluetooth: hci0: command 0x0c03 tx timeout
[**    ] (1 of 2) A start job is running for…b2d6-2108110dcc6d (13s / 1min 30s)
[ TIME ] Timed out waiting for device 1-0337-495f-b39e-3452d5e5c3ef.
[DEPEND] Dependency failed for File…1-0337-495f-b39e-3452d5e5c3ef.
[DEPEND] Dependency failed for /boot.
[DEPEND] Dependency failed for Local File Systems.
[ TIME ] Timed out waiting for device f-116f-46ae-b2d6-2108110dcc6d.
[DEPEND] Dependency failed for File…f-116f-46ae-b2d6-2108110dcc6d.
[DEPEND] Dependency failed for /home.
[  OK  ] Stopped Dispatch Password …ts to Console Directory Watch.
[  OK  ] Stopped Forward Password R…uests to Wall Directory Watch.
[  OK  ] Reached target Timers.
         Starting Packet Filtering Rules...
[  OK  ] Reached target Login Prompts.
[  OK  ] Reached target Sockets.
         Starting Load AppArmor profiles...
[  OK  ] Started Emergency Shell.
[  OK  ] Reached target Emergency Mode.
         Starting Create Volatile Files and Directories...
[  OK  ] Finished Create Volatile Files and Directories.
         Starting Update UTMP about System Boot/Shutdown...
[  OK  ] Finished Update UTMP about System Boot/Shutdown.
         Starting Update UTMP about System Runlevel Changes...
[  OK  ] Finished Update UTMP about System Runlevel Changes.
[   97.435542] audit: type=1400 audit(1629159395.699:2): apparmor="STATUS" operation="profile_load" profile="unconfined" name="lsb_release" pid=389 comm="apparmor_parser"
[   97.451041] audit: type=1400 audit(1629159395.707:3): apparmor="STATUS" operation="profile_load" profile="unconfined" name="nvidia_modprobe" pid=391 comm="apparmor_parser"
[   97.466776] audit: type=1400 audit(1629159395.707:4): apparmor="STATUS" operation="profile_load" profile="unconfined" name="nvidia_modprobe//kmod" pid=391 comm="apparmor_parser"
[  OK  ] Finished Packet Filtering Rules.
[  OK  ] Reached target Network (Pre).
[  OK  ] Reached target Network.
[   97.588358] audit: type=1400 audit(1629159395.851:5): apparmor="STATUS" operation="profile_load" profile="unconfined" name="/lib/systemd/systemd-logind" pid=392 comm="apparmor_parser"
[   97.628653] audit: type=1400 audit(1629159395.891:6): apparmor="STATUS" operation="profile_load" profile="unconfined" name="/usr/sbin/connmand" pid=390 comm="apparmor_parser"
[  OK  ] Finished Load AppArmor profiles.
You are in emergency mode. After logging in, type "journalctl -xb" to view
system logs, "systemctl reboot" to r
Cannot open access to console, the root account is locked.
See sulogin(8) man page for more details.

Press Enter to continue.

Reloading system manager configuration
Starting default target
You are in emergency mode. After logging in, type "journalctl -x
Cannot open access to console, the root account is locked.
See sulogin(8) man page for more details.

Press Enter to continue.

```