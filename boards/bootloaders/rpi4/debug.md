```sh
[    0.000027] EMMC: reset card.
[    0.000124] control1: 16143
[    0.002740] Divisor = 63, Freq Set = 396825
[    0.410153] CSD Contents : 00 40 0e 00 32 5b 59 00 00ee 9d 7f 80 0a 40 00
[    0.413981] cemmc_structure=1, spec_vers=0, taac=0x0E, nsac=0x00, tran_speed=0x32,ccc=0x05B5, read_bl_len=0x09, read_bl_partial=0b, write_blk_misalign=0b,read_blk_misalign=0b, dsr_imp=0b, sector_size =0x7F, erase_blk_en=1b
[    0.433612] CSD 2.0: ver2_c_size = 0xEFFD, card capacity: 32026132480 bytes or 32.03GiB
[    0.441517] wp_grp_size=0x0000000b, wp_grp_enable=0b, default_ecc=00b, r2w_factor=010b, write_bl_len=0x09, write_bl_partial=0b, file_format_grp=0, copy=0b, perm_write_protect=0b, tmp_write_protect=0b, file_format=0b ecc=00b
[    0.461242] control1: 271
[    0.463757] Divisor = 1, Freq Set = 25000000
[    0.468387] EMMC: Bus width set to 4
[    0.471411] EMMC: SD Card Type 2 HC, 30542Mb, mfr_id: 27, 'SM:EB2MW', r3.0, mfr_date: 8/2017, serial: 0xc8e6576d, RCA: 0x59b4
[    0.482609] EMMC2 driver initialized...

[    0.486433] rpi4 version 0.1.0
[    0.489385] Booting on: Raspberry Pi 4
[    0.493034] Current privilege level: EL1
[    0.496856] Exception handling state:
[    0.500418]       Debug:  Masked
[    0.503545]       SError: Masked
[    0.506673]       IRQ:    Masked
[    0.509800]       FIQ:    Masked
[    0.512928] Architectural timer resolution: 18 ns
[    0.517532] Drivers loaded:
[    0.520225]       1. BCM GPIO
[    0.523093]       2. BCM PL011 UART
[    0.526480] Chars written: 1483
[W   0.529523] wait duration smaller than architecturally supported, skipping
[    0.536297] create new emmc-fat controller...
[    0.543305]  Listing root directory:

[    0.545052]          Found: DirEntry { name: ShortFileName("boot"), mtime: Timestamp(2021-05-07 16:06:08), ctime: Timestamp(2021-05-07 16:06:08), attributes: FV, cluster: Cluster(0), size: 0, entry_block: BlockIdx(16290), entry_offset: 0 }
[    0.564630]          Found: DirEntry { name: ShortFileName("BCM271~1.DTB"), mtime: Timestamp(2022-01-16 13:09:50), ctime: Timestamp(2022-01-16 13:10:46), attributes: FA, cluster: Cluster(262146), size: 25934, entry_block: BlockIdx(16290), entry_offset: 96 }
[    0.586695]          Found: DirEntry { name: ShortFileName("KERNEL8.IMG"), mtime: Timestamp(2022-01-14 17:11:36), ctime: Timestamp(2022-01-14 17:05:22), attributes: FA, cluster: Cluster(458754), size: 125243392, entry_block: BlockIdx(16290), entry_offset: 128 }
[    0.609109]          Found: DirEntry { name: ShortFileName("VMLINUZ"), mtime: Timestamp(2022-01-08 18:37:12), ctime: Timestamp(2022-01-08 18:38:16), attributes: FA, cluster: Cluster(97567), size: 29272576, entry_block: BlockIdx(16290), entry_offset: 160 }
[    0.631001]          Found: DirEntry { name: ShortFileName("INITRA~1"), mtime: Timestamp(2022-01-14 16:21:24), ctime: Timestamp(2022-01-14 17:05:40), attributes: FA, cluster: Cluster(307254), size: 32901194, entry_block: BlockIdx(16290), entry_offset: 224 }
[    0.657248]          Found: DirEntry { name: ShortFileName("CONFIG.TXT"), mtime: Timestamp(2022-01-04 11:46:38), ctime: Timestamp(2021-05-07 15:07:00), attributes: FA, cluster: Cluster(1017), size: 1846, entry_block: BlockIdx(17103), entry_offset: 352 }
[    0.676113]          Found: DirEntry { name: ShortFileName("FIXUP4.DAT"), mtime: Timestamp(2021-04-30 14:01:38), ctime: Timestamp(2021-05-07 15:07:00), attributes: FA, cluster: Cluster(1036), size: 5446, entry_block: BlockIdx(17103), entry_offset: 480 }
[    0.700748]          Found: DirEntry { name: ShortFileName("START4.ELF"), mtime: Timestamp(2021-04-30 14:01:38), ctime: Timestamp(2021-05-07 15:07:02), attributes: FA, cluster: Cluster(59166), size: 2228768, entry_block: BlockIdx(54532), entry_offset: 224 }
[    0.721737]          Found: DirEntry { name: ShortFileName("SYSTEM~1"), mtime: Timestamp(2021-09-21 13:57:30), ctime: Timestamp(2021-09-21 13:57:28), attributes: DHS, cluster: Cluster(97564), size: 0, entry_block: BlockIdx(105365), entry_offset: 192 }
[    0.742424] Get handle to `dtb` file in root_dir...
[    0.747249]          load `dtb` into RAM...
[    0.793110]          loaded dtb: 25934 bytes, starting at addr: 0x3a00000
[    0.796329] Get handle to `kernel` file in root_dir...
[    0.802532]          load `kernel` into RAM...
[   45.722189]          loaded kernel: 29272576 bytes, starting at addr: 0x3c00000
[   45.726017] Get handle to `initramfs` file in root_dir...
[   45.732355]          load `initramfs` into RAM...
[   94.588911]          loaded initramfs: 32901194 bytes, starting at addr: 0x5890000

[   94.592999] ***************************************** Starting kernel ********************************************

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
[    0.000000] Kernel command line: root=UUID=f3f9f167-f409-d801-d058-f167f409d801 rootwait ro plymouth.ignore-serial-consoles fsck.mode=auto fsck.repair=yes cma=128M
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
[    0.000219] Console: colour dummy device 80x25
[    0.000589] printk: console [tty0] enabled
[    0.000687] Calibrating delay loop (skipped), value calculated using timer frequency.. 108.00 BogoMIPS (lpj=216000)
[    0.000717] pid_max: default: 32768 minimum: 301
[    0.000864] LSM: Security Framework initializing
[    0.000905] Yama: disabled by default; enable with sysctl kernel.yama.*
[    0.001059] AppArmor: AppArmor initialized
[    0.001080] TOMOYO Linux initialized
[    0.001222] Mount-cache hash table entries: 8192 (order: 4, 65536 bytes, linear)
[    0.001299] Mountpoint-cache hash table entries: 8192 (order: 4, 65536 bytes, linear)
[    0.003784] rcu: Hierarchical SRCU implementation.
[    0.006004] EFI services will not be available.
[    0.006563] smp: Bringing up secondary CPUs ...
[    0.007241] Detected PIPT I-cache on CPU1
[    0.007317] CPU1: Booted secondary processor 0x0000000001 [0x410fd083]
[    0.008159] Detected PIPT I-cache on CPU2
[    0.008206] CPU2: Booted secondary processor 0x0000000002 [0x410fd083]
[    0.009024] Detected PIPT I-cache on CPU3
[    0.009071] CPU3: Booted secondary processor 0x0000000003 [0x410fd083]
[    0.009180] smp: Brought up 1 node, 4 CPUs
[    0.009247] SMP: Total of 4 processors activated.
[    0.009261] CPU features: detected: 32-bit EL0 Support
[    0.009272] CPU features: detected: 32-bit EL1 Support
[    0.009286] CPU features: detected: CRC32 instructions
[    0.027603] ------------[ cut here ]------------
[    0.027636] CPU: CPUs started in inconsistent modes
[    0.027650] WARNING: CPU: 0 PID: 1 at arch/arm64/kernel/smp.c:426 smp_cpus_done+0x78/0xc4
[    0.027702] Modules linked in:
[    0.027720] CPU: 0 PID: 1 Comm: swapper/0 Not tainted 5.15.0-trunk-arm64 #1  Debian 5.15.1-1~exp1+apertis1
[    0.027744] Hardware name: Raspberry Pi 4 Model B (DT)
[    0.027757] pstate: 60000005 (nZCv daif -PAN -UAO -TCO -DIT -SSBS BTYPE=--)
[    0.027778] pc : smp_cpus_done+0x78/0xc4
[    0.027799] lr : smp_cpus_done+0x78/0xc4
[    0.027818] sp : ffff800011cabe00
[    0.027829] x29: ffff800011cabe00 x28: 0000000000000000 x27: 0000000000000000
[    0.027858] x26: 0000000000000000 x25: 0000000000000000 x24: 0000000000000000
[    0.027884] x23: 0000000000000000 x22: ffff800011bf1000 x21: 0000000000000000
[    0.027909] x20: ffff800011527e40 x19: ffff800011a20000 x18: 0000000000000001
[    0.027934] x17: 000000007ebb0ace x16: 00000000aa94fec1 x15: 0720072007200720
[    0.027959] x14: 0720072d072d072d x13: 7365646f6d20746e x12: 65747369736e6f63
[    0.027984] x11: ffff800011a00288 x10: ffff8000119a9588 x9 : ffff80001011cc1c
[    0.028009] x8 : ffff8000119a7b68 x7 : ffff8000119ffb68 x6 : fffffffffffe1418
[    0.028033] x5 : 0000000000001a20 x4 : 000000000000aff5 x3 : 0000000000000000
[    0.028057] x2 : 0000000000000000 x1 : 0000000000000000 x0 : ffff0000401b9e80
[    0.028082] Call trace:
[    0.028094]  smp_cpus_done+0x78/0xc4
[    0.028114]  smp_init+0x88/0x98
[    0.028136]  kernel_init_freeable+0x194/0x328
[    0.028157]  kernel_init+0x30/0x140
[    0.028174]  ret_from_fork+0x10/0x20
[    0.028200] ---[ end trace 7f3517c5d4de7f98 ]---
[    0.028306] alternatives: patching kernel code
[    0.118413] node 0 deferred pages initialised in 88ms
[    0.120025] devtmpfs: initialized
[    0.128418] Registered cp15_barrier emulation handler
[    0.128471] Registered setend emulation handler
[    0.128491] KASLR disabled due to lack of seed
[    0.128799] clocksource: jiffies: mask: 0xffffffff max_cycles: 0xffffffff, max_idle_ns: 7645041785100000 ns
[    0.128898] futex hash table entries: 1024 (order: 4, 65536 bytes, linear)
[    0.134141] pinctrl core: initialized pinctrl subsystem
[    0.134931] DMI not present or invalid.
[    0.135636] NET: Registered PF_NETLINK/PF_ROUTE protocol family
[    0.140658] DMA: preallocated 512 KiB GFP_KERNEL pool for atomic allocations
[    0.141480] DMA: preallocated 512 KiB GFP_KERNEL|GFP_DMA pool for atomic allocations
[    0.142482] DMA: preallocated 512 KiB GFP_KERNEL|GFP_DMA32 pool for atomic allocations
[    0.142586] audit: initializing netlink subsys (disabled)
[    0.142861] audit: type=2000 audit(0.140:1): state=initialized audit_enabled=0 res=1
[    0.144056] thermal_sys: Registered thermal governor 'fair_share'
[    0.144066] thermal_sys: Registered thermal governor 'bang_bang'
[    0.144083] thermal_sys: Registered thermal governor 'step_wise'
[    0.144096] thermal_sys: Registered thermal governor 'user_space'
[    0.144108] thermal_sys: Registered thermal governor 'power_allocator'
[    0.144335] cpuidle: using governor ladder
[    0.144379] cpuidle: using governor menu
[    0.144535] hw-breakpoint: found 6 breakpoint and 4 watchpoint registers.
[    0.144667] ASID allocator initialised with 65536 entries
[    0.145088] Serial: AMBA PL011 UART driver
[    0.171912] HugeTLB registered 1.00 GiB page size, pre-allocated 0 pages
[    0.171954] HugeTLB registered 32.0 MiB page size, pre-allocated 0 pages
[    0.171970] HugeTLB registered 2.00 MiB page size, pre-allocated 0 pages
[    0.171986] HugeTLB registered 64.0 KiB page size, pre-allocated 0 pages
[    0.831283] ACPI: Interpreter disabled.
[    0.831915] iommu: Default domain type: Translated
[    0.831935] iommu: DMA domain TLB invalidation policy: strict mode
[    0.832260] vgaarb: loaded
[    0.832667] EDAC MC: Ver: 3.0.0
[    0.834455] NetLabel: Initializing
[    0.834481] NetLabel:  domain hash size = 128
[    0.834493] NetLabel:  protocols = UNLABELED CIPSOv4 CALIPSO
[    0.834605] NetLabel:  unlabeled traffic allowed by default
[    0.835021] clocksource: Switched to clocksource arch_sys_counter
[    0.880572] VFS: Disk quotas dquot_6.6.0
[    0.880684] VFS: Dquot-cache hash table entries: 512 (order 0, 4096 bytes)
[    0.881520] AppArmor: AppArmor Filesystem Enabled
[    0.881751] pnp: PnP ACPI: disabled
[    0.891072] NET: Registered PF_INET protocol family
[    0.891639] IP idents hash table entries: 65536 (order: 7, 524288 bytes, linear)
[    0.894750] tcp_listen_portaddr_hash hash table entries: 2048 (order: 3, 32768 bytes, linear)
[    0.895092] TCP established hash table entries: 32768 (order: 6, 262144 bytes, linear)
[    0.895817] TCP bind hash table entries: 32768 (order: 7, 524288 bytes, linear)
[    0.896124] TCP: Hash tables configured (established 32768 bind 32768)
[    0.896609] MPTCP token hash table entries: 4096 (order: 4, 98304 bytes, linear)
[    0.896812] UDP hash table entries: 2048 (order: 4, 65536 bytes, linear)
[    0.896941] UDP-Lite hash table entries: 2048 (order: 4, 65536 bytes, linear)
[    0.897681] NET: Registered PF_UNIX/PF_LOCAL protocol family
[    0.897730] NET: Registered PF_XDP protocol family
[    0.897752] PCI: CLS 0 bytes, default 64
[    0.898449] Trying to unpack rootfs image as initramfs...
[    0.903855] hw perfevents: enabled with armv8_cortex_a72 PMU driver, 7 counters available
[    0.904057] kvm [1]: HYP mode not available
[    0.905437] Initialise system trusted keyrings
[    0.905494] Key type blacklist registered
[    0.905737] workingset: timestamp_bits=42 max_order=20 bucket_order=0
[    0.912163] zbud: loaded
[    0.913068] integrity: Platform Keyring initialized
[    0.913094] Key type asymmetric registered
[    0.913109] Asymmetric key parser 'x509' registered
[    0.913252] Block layer SCSI generic (bsg) driver version 0.4 loaded (major 248)
[    0.913534] io scheduler mq-deadline registered
[    0.922191] shpchp: Standard Hot Plug PCI Controller Driver version: 0.4
[    0.923146] brcm-pcie fd500000.pcie: host bridge /scb/pcie@7d500000 ranges:
[    0.923185] brcm-pcie fd500000.pcie:   No bus range found for /scb/pcie@7d500000, using [bus 00-ff]
[    0.923235] brcm-pcie fd500000.pcie:      MEM 0x0600000000..0x0603ffffff -> 0x00f8000000
[    0.923279] brcm-pcie fd500000.pcie:   IB MEM 0x0000000000..0x00bfffffff -> 0x0000000000
[    0.989111] brcm-pcie fd500000.pcie: link up, 5.0 GT/s PCIe x1 (SSC)
[    0.989423] brcm-pcie fd500000.pcie: PCI host bridge to bus 0000:00
[    0.989446] pci_bus 0000:00: root bus resource [bus 00-ff]
[    0.989467] pci_bus 0000:00: root bus resource [mem 0x600000000-0x603ffffff] (bus address [0xf8000000-0xfbffffff])
[    0.989528] pci 0000:00:00.0: [14e4:2711] type 01 class 0x060400
[    0.989626] pci 0000:00:00.0: PME# supported from D0 D3hot
[    0.992175] pci 0000:01:00.0: [1106:3483] type 00 class 0x0c0330
[    0.992256] pci 0000:01:00.0: reg 0x10: [mem 0x00000000-0x00000fff 64bit]
[    0.992449] pci 0000:01:00.0: PME# supported from D0 D3hot
[    1.005106] pci 0000:00:00.0: BAR 14: assigned [mem 0x600000000-0x6000fffff]
[    1.005155] pci 0000:01:00.0: BAR 0: assigned [mem 0x600000000-0x600000fff 64bit]
[    1.005188] pci 0000:00:00.0: PCI bridge to [bus 01]
[    1.005206] pci 0000:00:00.0:   bridge window [mem 0x600000000-0x6000fffff]
[    1.005448] pcieport 0000:00:00.0: enabling device (0000 -> 0002)
[    1.005646] pcieport 0000:00:00.0: PME: Signaling with IRQ 50
[    1.006017] pcieport 0000:00:00.0: AER: enabled with IRQ 50
[    1.017297] Serial: 8250/16550 driver, 4 ports, IRQ sharing enabled
[    1.019477] fe215040.serial: ttyS1 at MMIO 0xfe215040 (irq = 26, base_baud = 24999999) is a 16550
[    2.166896] printk: console [ttyS1] enabled
[    2.172336] Serial: AMBA driver
[    2.175614] SuperH (H)SCI(F) driver initialized
[    2.180842] msm_serial: driver initialized
[    2.186089] cacheinfo: Unable to detect cache hierarchy for CPU 0
[    2.193210] bcm2835-power bcm2835-power: Broadcom BCM2835 power domains driver
[    2.201600] mousedev: PS/2 mouse device common for all mice
[    2.208505] brcmstb-i2c fef04500.i2c:  @97500hz registered in polling mode
[    2.215845] brcmstb-i2c fef09500.i2c:  @97500hz registered in polling mode
[    2.224295] ledtrig-cpu: registered to indicate activity on CPUs
[    2.231368] bcm2835-mbox fe00b880.mailbox: mailbox enabled
[    2.239179] NET: Registered PF_INET6 protocol family
[    3.341668] Freeing initrd memory: 32128K
[    3.383764] Segment Routing with IPv6
[    3.387584] In-situ OAM (IOAM) with IPv6
[    3.391690] mip6: Mobile IPv6
[    3.394713] NET: Registered PF_PACKET protocol family
[    3.400029] mpls_gso: MPLS GSO support
[    3.404642] registered taskstats version 1
[    3.408843] Loading compiled-in X.509 certificates
[    3.564178] Loaded X.509 cert 'Debian Secure Boot CA: 6ccece7e4c6c0d1f6149f3dd27dfcc5cbb419ea1'
[    3.573121] Loaded X.509 cert 'Debian Secure Boot Signer 2021 - linux: 4b6ef5abca669825178e052c84667ccbc0531f8c'
[    3.584481] zswap: loaded using pool lzo/zbud
[    3.589844] Key type ._fscrypt registered
[    3.593950] Key type .fscrypt registered
[    3.597948] Key type fscrypt-provisioning registered
[    3.620600] Key type encrypted registered
[    3.624747] AppArmor: AppArmor sha1 policy hashing enabled
[    3.630373] ima: No TPM chip found, activating TPM-bypass!
[    3.635990] ima: Allocated hash algorithm: sha256
[    3.640833] ima: No architecture policies found
[    3.645500] evm: Initialising EVM extended attributes:
[    3.650735] evm: security.selinux
[    3.654113] evm: security.SMACK64 (disabled)
[    3.658462] evm: security.SMACK64EXEC (disabled)
[    3.663163] evm: security.SMACK64TRANSMUTE (disabled)
[    3.668304] evm: security.SMACK64MMAP (disabled)
[    3.673004] evm: security.apparmor
[    3.676466] evm: security.ima
[    3.679487] evm: security.capability
[    3.683127] evm: HMAC attrs: 0x1
[    3.692037] fe201000.serial: ttyAMA0 at MMIO 0xfe201000 (irq = 24, base_baud = 0) is a PL011 rev2
[    3.701373] serial serial0: tty port ttyAMA0 registered
[    3.707873] raspberrypi-firmware soc:firmware: Attached to firmware from 2021-04-30T13:45:52
[    3.891211] Freeing unused kernel memory: 5760K
[    3.953136] Checked W+X mappings: passed, no W+X pages found
[    3.958952] Run /init as init process
Loading, please wait...
Starting version 247.3-6+apertis1bv2022dev3b2
[    4.331289] phy_generic: module verification failed: signature and/or required key missing - tainting kernel
[    4.413765] usb_phy_generic phy: supply vcc not found, using dummy regulator
[    4.426540] libphy: Fixed MDIO Bus: probed
[    4.434784] usbcore: registered new interface driver usbfs
[    4.440532] usbcore: registered new interface driver hub
[    4.440781] sdhci: Secure Digital Host Controller Interface driver
[    4.446169] usbcore: registered new device driver usb
[    4.447332] bcmgenet fd580000.ethernet: GENET 5.0 EPHY: 0x0000
[    4.447370] bcmgenet fd580000.ethernet: using random Ethernet MAC
[    4.452361] sdhci: Copyright(c) Pierre Ossman
[    4.458088] sdhci-pltfm: SDHCI platform and OF driver helper
[    4.467096] libphy: bcmgenet MII bus: probed
[    4.481304] sdhci-iproc fe300000.mmc: allocated mmc-pwrseq
[    4.527168] mmc0: SDHCI controller on fe340000.mmc [fe340000.mmc] using ADMA
[    4.545239] xhci_hcd 0000:01:00.0: xHCI Host Controller
[    4.550932] xhci_hcd 0000:01:00.0: new USB bus registered, assigned bus number 1
[    4.559404] xhci_hcd 0000:01:00.0: hcc params 0x002841eb hci version 0x100 quirks 0x0000040000000890
[    4.569754] usb usb1: New USB device found, idVendor=1d6b, idProduct=0002, bcdDevice= 5.15
[    4.572374] dwc2 fe980000.usb: supply vusb_d not found, using dummy regulator
[    4.578225] usb usb1: New USB device strings: Mfr=3, Product=2, SerialNumber=1
[    4.578236] usb usb1: Product: xHCI Host Controller
[    4.578244] usb usb1: Manufacturer: Linux 5.15.0-trunk-arm64 xhci-hcd
[    4.578251] usb usb1: SerialNumber: 0000:01:00.0
[    4.578396] unimac-mdio unimac-mdio.-19: Broadcom UniMAC MDIO bus
[    4.578948] hub 1-0:1.0: USB hub found
[    4.579105] hub 1-0:1.0: 1 port detected
[    4.579759] xhci_hcd 0000:01:00.0: xHCI Host Controller
[    4.579781] xhci_hcd 0000:01:00.0: new USB bus registered, assigned bus number 2
[    4.579801] xhci_hcd 0000:01:00.0: Host supports USB 3.0 SuperSpeed
[    4.580061] usb usb2: New USB device found, idVendor=1d6b, idProduct=0003, bcdDevice= 5.15
[    4.580075] usb usb2: New USB device strings: Mfr=3, Product=2, SerialNumber=1
[    4.580084] usb usb2: Product: xHCI Host Controller
[    4.580091] usb usb2: Manufacturer: Linux 5.15.0-trunk-arm64 xhci-hcd
[    4.580098] usb usb2: SerialNumber: 0000:01:00.0
[    4.580708] hub 2-0:1.0: USB hub found
[    4.580752] hub 2-0:1.0: 4 ports detected
[    4.586527] dwc2 fe980000.usb: supply vusb_a not found, using dummy regulator
[    4.593354] mmc1: SDHCI controller on fe300000.mmc [fe300000.mmc] using PIO
[    4.699129] dwc2 fe980000.usb: EPs: 8, dedicated fifos, 4080 entries in SPRAM
Begin: Loading essential drivers ... [    4.747554] random: fast init done
[    4.761425] mmc0: new ultra high speed DDR50 SDHC card at address 59b4
[    4.769405] mmcblk0: mmc0:59b4 EB2MW 29.8 GiB
[    4.777420]  mmcblk0: p1 p2
[    4.791325] mmc1: new high speed SDIO card at address 0001
done.
Begin: Running /scripts/init-premount ... done.
Begin: Mounting root file system ... Be[    4.835072] usb 1-1: new high-speed USB device number 2 using xhci_hcd
gin: Running /scripts/local-top ... done.
Begin: Running /scripts/local-premount ... done.
Begin: Will now check root file system ... fsck from util-linux 2.36.1
[    4.989661] usb 1-1: New USB device found, idVendor=2109, idProduct=3431, bcdDevice= 4.21
[    4.998086] usb 1-1: New USB device strings: Mfr=0, Product=1, SerialNumber=0
[    5.005375] usb 1-1: Product: USB2.0 Hub
[    5.012512] hub 1-1:1.0: USB hub found
[    5.016637] hub 1-1:1.0: 4 ports detected
[/sbin/fsck.ext4 (1) -- /dev/mmcblk0p2] fsck.ext4 -y -C0 /dev/mmcblk0p2
e2fsck 1.46.2 (28-Feb-2021)
Backing up journal inode block information.

system: ignoring check interval, broken_system_clock set
system: clean, 11/807380 files, 211012/3228672 blocks
done.
[    5.686979] EXT4-fs (mmcblk0p2): mounted filesystem with ordered data mode. Opts: (null). Quota mode: none.
done.
Begin: Running /scripts/local-bottom ... done.
Begin: Running /scripts/init-bottom ... mount: mounting /dev on /root/dev failed: No such file or directory
mount: mounting /dev on /root/dev failed: No such file or directory
done.
mount: mounting /run on /root/run failed: No such file or directory
run-init: can't execute '/sbin/init': No such file or directory
Target filesystem doesn't have requested /sbin/init.
run-init: can't execute '/sbin/init': No such file or directory
run-init: can't execute '/etc/init': No such file or directory
run-init: can't execute '/bin/init': No such file or directory
run-init: can't execute '/bin/sh': No such file or directory
run-init: can't execute '': No such file or directory
No init found. Try passing init= bootarg.


BusyBox v1.30.1 (Apertis 1:1.30.1-6+apertis2bv2022dev3b1) built-in shell (ash)
Enter 'help' for a list of built-in commands.

(initramfs) help
Built-in commands:
------------------
        . : [ [[ alias bg break cd chdir command continue echo eval exec
        exit export false fg getopts hash help history jobs kill let
        local printf pwd read readonly return set shift source test times
        trap true type ulimit umask unalias unset wait
(initramfs) ls
bin      dev      init     proc     run      scripts  tmp      var
conf     etc      lib      root     sbin     sys      usr

```