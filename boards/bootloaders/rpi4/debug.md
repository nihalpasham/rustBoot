```powershell
boards\bootloaders\rpi4 on  main [✘!?] is 📦 v0.1.0 via 🦀 v1.61.0-nightly
❯ terminal-s.exe
--- COM3 is connected. Press Ctrl+] to quit ---
[    2.017038] EMMC: reset card.
[    2.017066] control1: 16143
[    2.019742] Divisor = 63, Freq Set = 396825
[    2.427116] CSD Contents : 00 40 0e 00 32 5b 59 00 00ed c8 7f 80 0a 40 40
[    2.430933] cemmc_structure=1, spec_vers=0, taac=0x0E, nsac=0x00, tran_speed=0x32,ccc=0x05B5, read_bl_len=0x09, read_bl_partial=0b, write_blk_misalign=0b,read_blk_misalign=0b, dsr_imp=0b, sector_size =0x7F, erase_blk_en=1b
[    2.450566] CSD 2.0: ver2_c_size = 0xEFFC, card capacity: 31914459136 bytes or 31.91GiB
[    2.458471] wp_grp_size=0x0000000b, wp_grp_enable=0b, default_ecc=00b, r2w_factor=010b, write_bl_len=0x09, write_bl_partial=0b, file_format_grp=0, copy=1b, perm_write_protect=0b, tmp_write_protect=0b, file_format=0b ecc=00b
[    2.478192] control1: 271
[    2.480712] Divisor = 1, Freq Set = 25000000
[    2.487196] EMMC: Bus width set to 4
[    2.488359] EMMC: SD Card Type 2 HC, 30436Mb, mfr_id: 3, 'SD:ACLCD', r8.0, mfr_date: 1/2017, serial: 0xbbce119c, RCA: 0xaaaa
[    2.499476] EMMC2 driver initialized...

[    2.503299] rpi4 version 0.1.0
[    2.506252] Booting on: Raspberry Pi 4
[    2.509901] MMU online. Special regions:
[    2.513724]       0x00080000 - 0x000a3fff | 144 KiB | C   RO PX  | Kernel code and RO data
[    2.521890]       0x1fff0000 - 0x1fffffff |  64 KiB | Dev RW PXN | Remapped Device MMIO
[    2.529796]       0xfe000000 - 0xff84ffff |  24 MiB | Dev RW PXN | Device MMIO
[    2.536920] Current privilege level: EL1
[    2.540742] Exception handling state:
[    2.544304]       Debug:  Masked
[    2.547431]       SError: Masked
[    2.550559]       IRQ:    Masked
[    2.553686]       FIQ:    Masked
[    2.556814] Architectural timer resolution: 18 ns
[    2.561418] Drivers loaded:
[    2.564111]       1. BCM GPIO
[    2.566978]       2. BCM PL011 UART
[    2.570366] Chars written: 1793
[    2.573409] [INFO]  create new emmc-fat controller...
[    2.579141]          - rustBoot::fs::controller @ line:200
[    2.588591] Listing root directory:
[    2.593325]      - Found: SIGNED~1.ITB
[    2.594883] loading fit-image...
[   34.285615] loaded fit: 62202019 bytes, starting at addr: 0x4200000
[   34.288911] authenticating fit-image...
[   34.293908] [INFO]  computing "kernel" hash
[   34.298641]          - rustBoot::dt::fit @ line:289
[   35.404489] [INFO]  computed "kernel" hash: 97dcbff24ad0a60514e31a7a6b34a765681fea81f8dd11e4644f3ec81e1044fb
[   35.412127]          - rustBoot::dt::fit @ line:294
[   35.416913] [INFO]  kernel integrity consistent with supplied itb...
[   35.424724]          - rustBoot::dt::fit @ line:308
[   35.429526] [INFO]  computing "fdt" hash
[   35.434107]          - rustBoot::dt::fit @ line:289
[   35.439856] [INFO]  computed "fdt" hash: 3572783be74511b710ed7fca9b3131e97fd8073c620a94269a4e4ce79d331540
[   35.449136]          - rustBoot::dt::fit @ line:294
[   35.453920] [INFO]  fdt integrity consistent with supplied itb...
[   35.461472]          - rustBoot::dt::fit @ line:308
[   35.466274] [INFO]  computing "ramdisk" hash
[   35.471202]          - rustBoot::dt::fit @ line:289
[   36.713353] [INFO]  computed "ramdisk" hash: f1290587e2155e3a5c2c870fa1d6e3e2252fb0dddf74992113d2ed86bc67f37c
[   36.721078]          - rustBoot::dt::fit @ line:294
[   36.725862] [INFO]  ramdisk integrity consistent with supplied itb...
[   36.733762]          - rustBoot::dt::fit @ line:308
[   36.738568] [INFO]  computing "rbconfig" hash
[   36.743579]          - rustBoot::dt::fit @ line:289
[   36.748367] [INFO]  computed "rbconfig" hash: b16d058c4f09abdb8da98561f3a15d06ff271c38a4655c2be11dec23567fd519
[   36.759042]          - rustBoot::dt::fit @ line:294
[   36.763825] [INFO]  rbconfig integrity consistent with supplied itb...
[   36.771813]          - rustBoot::dt::fit @ line:308
######## ecdsa signature checks out, image is authentic ########
[   36.808898] relocating kernel to addr: 0x2200000
[   36.832642] relocating initrd to addr: 0x200000
[   36.834200] load rbconfig...
[   36.837013] patching dtb...
[   36.840865] relocating dtb to addr: 0x4000000
***************************************** Starting kernel ********************************************
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
[    0.000000] cma: Reserved 4 MiB at 0x000000003b000000
[    0.000000] percpu: Embedded 29 pages/cpu s81752 r8192 d28840 u118784
[    0.000000] Detected PIPT I-cache on CPU0
[    0.000000] CPU features: detected: Spectre-v2
[    0.000000] CPU features: detected: Spectre-v3a
[    0.000000] CPU features: detected: Spectre-v4
[    0.000000] CPU features: detected: ARM errata 1165522, 1319367, or 1530923
[    0.000000] Built 1 zonelists, mobility grouping on.  Total pages: 996912
[    0.000000] Policy zone: DMA32
[    0.000000] Kernel command line: root=UUID=64bc182a-ca9d-4aa1-8936-d2919863c22a rootwait ro plymouth.ignore-serial-consoles fsck.mode=auto fsck.repair=yes cma=128
[    0.000000] Dentry cache hash table entries: 524288 (order: 10, 4194304 bytes, linear)
[    0.000000] Inode-cache hash table entries: 262144 (order: 9, 2097152 bytes, linear)
[    0.000000] mem auto-init: stack:off, heap alloc:on, heap free:off
[    0.000000] software IO TLB: mapped [mem 0x0000000037000000-0x000000003b000000] (64MB)
[    0.000000] Memory: 970864K/4050944K available (12352K kernel code, 2538K rwdata, 7808K rodata, 5760K init, 622K bss, 208032K reserved, 4096K cma-reserved)
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
[    0.000218] Console: colour dummy device 80x25
[    0.000587] printk: console [tty0] enabled
[    0.000687] Calibrating delay loop (skipped), value calculated using timer frequency.. 108.00 BogoMIPS (lpj=216000)
[    0.000716] pid_max: default: 32768 minimum: 301
[    0.000864] LSM: Security Framework initializing
[    0.000904] Yama: disabled by default; enable with sysctl kernel.yama.*
[    0.001056] AppArmor: AppArmor initialized
[    0.001077] TOMOYO Linux initialized
[    0.001218] Mount-cache hash table entries: 8192 (order: 4, 65536 bytes, linear)
[    0.001295] Mountpoint-cache hash table entries: 8192 (order: 4, 65536 bytes, linear)
[    0.003775] rcu: Hierarchical SRCU implementation.
[    0.005981] EFI services will not be available.
[    0.006545] smp: Bringing up secondary CPUs ...
[    0.007225] Detected PIPT I-cache on CPU1
[    0.007301] CPU1: Booted secondary processor 0x0000000001 [0x410fd083]
[    0.008145] Detected PIPT I-cache on CPU2
[    0.008192] CPU2: Booted secondary processor 0x0000000002 [0x410fd083]
[    0.009009] Detected PIPT I-cache on CPU3
[    0.009056] CPU3: Booted secondary processor 0x0000000003 [0x410fd083]
[    0.009166] smp: Brought up 1 node, 4 CPUs
[    0.009233] SMP: Total of 4 processors activated.
[    0.009246] CPU features: detected: 32-bit EL0 Support
[    0.009257] CPU features: detected: 32-bit EL1 Support
[    0.009271] CPU features: detected: CRC32 instructions
[    0.027606] ------------[ cut here ]------------
[    0.027641] CPU: CPUs started in inconsistent modes
[    0.027655] WARNING: CPU: 0 PID: 1 at arch/arm64/kernel/smp.c:426 smp_cpus_done+0x78/0xc4
[    0.027706] Modules linked in:
[    0.027724] CPU: 0 PID: 1 Comm: swapper/0 Not tainted 5.15.0-trunk-arm64 #1  Debian 5.15.1-1~exp1+apertis1
[    0.027749] Hardware name: Raspberry Pi 4 Model B (DT)
[    0.027761] pstate: 60000005 (nZCv daif -PAN -UAO -TCO -DIT -SSBS BTYPE=--)
[    0.027782] pc : smp_cpus_done+0x78/0xc4
[    0.027803] lr : smp_cpus_done+0x78/0xc4
[    0.027822] sp : ffff800011cabe00
[    0.027833] x29: ffff800011cabe00 x28: 0000000000000000 x27: 0000000000000000
[    0.027861] x26: 0000000000000000 x25: 0000000000000000 x24: 0000000000000000
[    0.027888] x23: 0000000000000000 x22: ffff800011bf1000 x21: 0000000000000000
[    0.027913] x20: ffff800011527e40 x19: ffff800011a20000 x18: 0000000000000001
[    0.027939] x17: 000000004d765e39 x16: 0000000058dcdf06 x15: 0720072007200720
[    0.027964] x14: 0720072d072d072d x13: 7365646f6d20746e x12: 65747369736e6f63
[    0.027989] x11: ffff800011a00288 x10: ffff8000119a9588 x9 : ffff80001011cc1c
[    0.028014] x8 : ffff8000119a7b68 x7 : ffff8000119ffb68 x6 : fffffffffffe1418
[    0.028038] x5 : 0000000000001a20 x4 : 000000000000aff5 x3 : 0000000000000000
[    0.028063] x2 : 0000000000000000 x1 : 0000000000000000 x0 : ffff0000401bbd00
[    0.028088] Call trace:
[    0.028100]  smp_cpus_done+0x78/0xc4
[    0.028120]  smp_init+0x88/0x98
[    0.028142]  kernel_init_freeable+0x194/0x328
[    0.028163]  kernel_init+0x30/0x140
[    0.028180]  ret_from_fork+0x10/0x20
[    0.028206] ---[ end trace ff88d5340271724e ]---
[    0.028313] alternatives: patching kernel code
[    0.118034] node 0 deferred pages initialised in 88ms
[    0.119655] devtmpfs: initialized
[    0.128051] Registered cp15_barrier emulation handler
[    0.128102] Registered setend emulation handler
[    0.128121] KASLR disabled due to lack of seed
[    0.128422] clocksource: jiffies: mask: 0xffffffff max_cycles: 0xffffffff, max_idle_ns: 7645041785100000 ns
[    0.128522] futex hash table entries: 1024 (order: 4, 65536 bytes, linear)
[    0.129080] pinctrl core: initialized pinctrl subsystem
[    0.129812] DMI not present or invalid.
[    0.130452] NET: Registered PF_NETLINK/PF_ROUTE protocol family
[    0.135674] DMA: preallocated 512 KiB GFP_KERNEL pool for atomic allocations
[    0.136535] DMA: preallocated 512 KiB GFP_KERNEL|GFP_DMA pool for atomic allocations
[    0.137674] DMA: preallocated 512 KiB GFP_KERNEL|GFP_DMA32 pool for atomic allocations
[    0.137789] audit: initializing netlink subsys (disabled)
[    0.138069] audit: type=2000 audit(0.136:1): state=initialized audit_enabled=0 res=1
[    0.139215] thermal_sys: Registered thermal governor 'fair_share'
[    0.139224] thermal_sys: Registered thermal governor 'bang_bang'
[    0.139242] thermal_sys: Registered thermal governor 'step_wise'
[    0.139254] thermal_sys: Registered thermal governor 'user_space'
[    0.139266] thermal_sys: Registered thermal governor 'power_allocator'
[    0.139529] cpuidle: using governor ladder
[    0.139575] cpuidle: using governor menu
[    0.139733] hw-breakpoint: found 6 breakpoint and 4 watchpoint registers.
[    0.139865] ASID allocator initialised with 65536 entries
[    0.140297] Serial: AMBA PL011 UART driver
[    0.167056] HugeTLB registered 1.00 GiB page size, pre-allocated 0 pages
[    0.167099] HugeTLB registered 32.0 MiB page size, pre-allocated 0 pages
[    0.167116] HugeTLB registered 2.00 MiB page size, pre-allocated 0 pages
[    0.167131] HugeTLB registered 64.0 KiB page size, pre-allocated 0 pages
[    0.827068] ACPI: Interpreter disabled.
[    0.827713] iommu: Default domain type: Translated
[    0.827732] iommu: DMA domain TLB invalidation policy: strict mode
[    0.828057] vgaarb: loaded
[    0.828465] EDAC MC: Ver: 3.0.0
[    0.830223] NetLabel: Initializing
[    0.830245] NetLabel:  domain hash size = 128
[    0.830257] NetLabel:  protocols = UNLABELED CIPSOv4 CALIPSO
[    0.830336] NetLabel:  unlabeled traffic allowed by default
[    0.830719] clocksource: Switched to clocksource arch_sys_counter
[    0.876449] VFS: Disk quotas dquot_6.6.0
[    0.876554] VFS: Dquot-cache hash table entries: 512 (order 0, 4096 bytes)
[    0.877438] AppArmor: AppArmor Filesystem Enabled
[    0.877669] pnp: PnP ACPI: disabled
[    0.886476] NET: Registered PF_INET protocol family
[    0.887077] IP idents hash table entries: 65536 (order: 7, 524288 bytes, linear)
[    0.890184] tcp_listen_portaddr_hash hash table entries: 2048 (order: 3, 32768 bytes, linear)
[    0.890559] TCP established hash table entries: 32768 (order: 6, 262144 bytes, linear)
[    0.891211] TCP bind hash table entries: 32768 (order: 7, 524288 bytes, linear)
[    0.891512] TCP: Hash tables configured (established 32768 bind 32768)
[    0.892017] MPTCP token hash table entries: 4096 (order: 4, 98304 bytes, linear)
[    0.892221] UDP hash table entries: 2048 (order: 4, 65536 bytes, linear)
[    0.892345] UDP-Lite hash table entries: 2048 (order: 4, 65536 bytes, linear)
[    0.893041] NET: Registered PF_UNIX/PF_LOCAL protocol family
[    0.893096] NET: Registered PF_XDP protocol family
[    0.893119] PCI: CLS 0 bytes, default 64
[    0.893884] Trying to unpack rootfs image as initramfs...
[    0.900468] hw perfevents: enabled with armv8_cortex_a72 PMU driver, 7 counters available
[    0.900707] kvm [1]: HYP mode not available
[    0.902192] Initialise system trusted keyrings
[    0.902254] Key type blacklist registered
[    0.902534] workingset: timestamp_bits=42 max_order=20 bucket_order=0
[    0.908980] zbud: loaded
[    0.909931] integrity: Platform Keyring initialized
[    0.909961] Key type asymmetric registered
[    0.909976] Asymmetric key parser 'x509' registered
[    0.910103] Block layer SCSI generic (bsg) driver version 0.4 loaded (major 248)
[    0.910412] io scheduler mq-deadline registered
[    0.918700] shpchp: Standard Hot Plug PCI Controller Driver version: 0.4
[    0.919683] brcm-pcie fd500000.pcie: host bridge /scb/pcie@7d500000 ranges:
[    0.919720] brcm-pcie fd500000.pcie:   No bus range found for /scb/pcie@7d500000, using [bus 00-ff]
[    0.919770] brcm-pcie fd500000.pcie:      MEM 0x0600000000..0x0603ffffff -> 0x00f8000000
[    0.919813] brcm-pcie fd500000.pcie:   IB MEM 0x0000000000..0x00bfffffff -> 0x0000000000
[    0.984807] brcm-pcie fd500000.pcie: link up, 5.0 GT/s PCIe x1 (SSC)
[    0.985126] brcm-pcie fd500000.pcie: PCI host bridge to bus 0000:00
[    0.985149] pci_bus 0000:00: root bus resource [bus 00-ff]
[    0.985170] pci_bus 0000:00: root bus resource [mem 0x600000000-0x603ffffff] (bus address [0xf8000000-0xfbffffff])
[    0.985230] pci 0000:00:00.0: [14e4:2711] type 01 class 0x060400
[    0.985329] pci 0000:00:00.0: PME# supported from D0 D3hot
[    0.987862] pci 0000:01:00.0: [1106:3483] type 00 class 0x0c0330
[    0.987945] pci 0000:01:00.0: reg 0x10: [mem 0x00000000-0x00000fff 64bit]
[    0.988137] pci 0000:01:00.0: PME# supported from D0 D3hot
[    1.000801] pci 0000:00:00.0: BAR 14: assigned [mem 0x600000000-0x6000fffff]
[    1.000848] pci 0000:01:00.0: BAR 0: assigned [mem 0x600000000-0x600000fff 64bit]
[    1.000881] pci 0000:00:00.0: PCI bridge to [bus 01]
[    1.000900] pci 0000:00:00.0:   bridge window [mem 0x600000000-0x6000fffff]
[    1.001164] pcieport 0000:00:00.0: enabling device (0000 -> 0002)
[    1.001349] pcieport 0000:00:00.0: PME: Signaling with IRQ 50
[    1.001736] pcieport 0000:00:00.0: AER: enabled with IRQ 50
[    1.013033] Serial: 8250/16550 driver, 4 ports, IRQ sharing enabled
[    1.015179] fe215040.serial: ttyS1 at MMIO 0xfe215040 (irq = 26, base_baud = 24999999) is a 16550
[    2.162191] printk: console [ttyS1] enabled
[    2.167671] Serial: AMBA driver
[    2.170930] SuperH (H)SCI(F) driver initialized
[    2.176151] msm_serial: driver initialized
[    2.181388] cacheinfo: Unable to detect cache hierarchy for CPU 0
[    2.188506] bcm2835-power bcm2835-power: Broadcom BCM2835 power domains driver
[    2.196904] mousedev: PS/2 mouse device common for all mice
[    2.203839] brcmstb-i2c fef04500.i2c:  @97500hz registered in polling mode
[    2.211178] brcmstb-i2c fef09500.i2c:  @97500hz registered in polling mode
[    2.219521] ledtrig-cpu: registered to indicate activity on CPUs
[    2.226532] bcm2835-mbox fe00b880.mailbox: mailbox enabled
[    2.234342] NET: Registered PF_INET6 protocol family
[    3.338864] Freeing initrd memory: 32128K
[    3.380855] Segment Routing with IPv6
[    3.384672] In-situ OAM (IOAM) with IPv6
[    3.388776] mip6: Mobile IPv6
[    3.391809] NET: Registered PF_PACKET protocol family
[    3.397146] mpls_gso: MPLS GSO support
[    3.401742] registered taskstats version 1
[    3.405941] Loading compiled-in X.509 certificates
[    3.561056] Loaded X.509 cert 'Debian Secure Boot CA: 6ccece7e4c6c0d1f6149f3dd27dfcc5cbb419ea1'
[    3.570002] Loaded X.509 cert 'Debian Secure Boot Signer 2021 - linux: 4b6ef5abca669825178e052c84667ccbc0531f8c'
[    3.581395] zswap: loaded using pool lzo/zbud
[    3.586701] Key type ._fscrypt registered
[    3.590820] Key type .fscrypt registered
[    3.594817] Key type fscrypt-provisioning registered
[    3.617312] Key type encrypted registered
[    3.621453] AppArmor: AppArmor sha1 policy hashing enabled
[    3.627080] ima: No TPM chip found, activating TPM-bypass!
[    3.632694] ima: Allocated hash algorithm: sha256
[    3.637547] ima: No architecture policies found
[    3.642217] evm: Initialising EVM extended attributes:
[    3.647452] evm: security.selinux
[    3.650828] evm: security.SMACK64 (disabled)
[    3.655174] evm: security.SMACK64EXEC (disabled)
[    3.659873] evm: security.SMACK64TRANSMUTE (disabled)
[    3.665014] evm: security.SMACK64MMAP (disabled)
[    3.669713] evm: security.apparmor
[    3.673174] evm: security.ima
[    3.676195] evm: security.capability
[    3.679834] evm: HMAC attrs: 0x1
[    3.688747] fe201000.serial: ttyAMA0 at MMIO 0xfe201000 (irq = 24, base_baud = 0) is a PL011 rev2
[    3.698104] serial serial0: tty port ttyAMA0 registered
[    3.704539] raspberrypi-firmware soc:firmware: Attached to firmware from 2022-01-20T13:56:48
[    3.789652] Freeing unused kernel memory: 5760K
[    3.855976] Checked W+X mappings: passed, no W+X pages found
[    3.861800] Run /init as init process
Loading, please wait...
Starting version 247.3-6+apertis1bv2022dev3b2
[    4.238970] phy_generic: module verification failed: signature and/or required key missing - tainting kernel
[    4.355776] usb_phy_generic phy: supply vcc not found, using dummy regulator
[    4.356792] sdhci: Secure Digital Host Controller Interface driver
[    4.369394] sdhci: Copyright(c) Pierre Ossman
[    4.388871] sdhci-pltfm: SDHCI platform and OF driver helper
[    4.402006] sdhci-iproc fe300000.mmc: allocated mmc-pwrseq
[    4.420285] libphy: Fixed MDIO Bus: probed
[    4.445277] usbcore: registered new interface driver usbfs
[    4.451076] usbcore: registered new interface driver hub
[    4.456700] usbcore: registered new device driver usb
[    4.457737] bcmgenet fd580000.ethernet: GENET 5.0 EPHY: 0x0000
[    4.463449] mmc1: SDHCI controller on fe340000.mmc [fe340000.mmc] using ADMA
[    4.468072] bcmgenet fd580000.ethernet: using random Ethernet MAC
[    4.486763] mmc0: SDHCI controller on fe300000.mmc [fe300000.mmc] using PIO
[    4.494758] libphy: bcmgenet MII bus: probed
[    4.531523] xhci_hcd 0000:01:00.0: xHCI Host Controller
[    4.537051] xhci_hcd 0000:01:00.0: new USB bus registered, assigned bus number 1
[    4.545517] xhci_hcd 0000:01:00.0: hcc params 0x002841eb hci version 0x100 quirks 0x0000040000000890
[    4.556068] usb usb1: New USB device found, idVendor=1d6b, idProduct=0002, bcdDevice= 5.15
[    4.562853] unimac-mdio unimac-mdio.-19: Broadcom UniMAC MDIO bus
[    4.564567] usb usb1: New USB device strings: Mfr=3, Product=2, SerialNumber=1
[    4.564681] dwc2 fe980000.usb: supply vusb_d not found, using dummy regulator
[    4.564861] dwc2 fe980000.usb: supply vusb_a not found, using dummy regulator
[    4.592631] usb usb1: Product: xHCI Host Controller
[    4.597650] usb usb1: Manufacturer: Linux 5.15.0-trunk-arm64 xhci-hcd
[    4.604248] usb usb1: SerialNumber: 0000:01:00.0
[    4.609951] hub 1-0:1.0: USB hub found
[    4.613934] hub 1-0:1.0: 1 port detected
[    4.618863] xhci_hcd 0000:01:00.0: xHCI Host Controller
[    4.624363] xhci_hcd 0000:01:00.0: new USB bus registered, assigned bus number 2
[    4.631985] xhci_hcd 0000:01:00.0: Host supports USB 3.0 SuperSpeed
[    4.638696] usb usb2: New USB device found, idVendor=1d6b, idProduct=0003, bcdDevice= 5.15
[    4.641515] random: fast init done
[    4.647166] usb usb2: New USB device strings: Mfr=3, Product=2, SerialNumber=1
[    4.657962] usb usb2: Product: xHCI Host Controller
[    4.662934] usb usb2: Manufacturer: Linux 5.15.0-trunk-arm64 xhci-hcd
[    4.666779] dwc2 fe980000.usb: EPs: 8, dedicated fifos, 4080 entries in SPRAM
[    4.668368] mmc0: new high speed SDIO card at address 0001
[    4.669500] usb usb2: SerialNumber: 0000:01:00.0
[    4.687775] hub 2-0:1.0: USB hub found
[    4.691797] hub 2-0:1.0: 4 ports detected
Begin: Loading essential drivers ... [    4.735126] mmc1: new ultra high speed DDR50 SDHC card at address aaaa
[    4.743392] mmcblk1: mmc1:aaaa ACLCD 29.7 GiB
[    4.754509]  mmcblk1: p1 p2
done.
Begin: Running /scripts/init-premount ... done.
Begin: Mounting root file system ... Begin: Running /scripts/local-top ... done.
Begin: Running /scripts/local-premount ... done.
Begin: Will now check root file system ... fsck from util-linux 2.36.1
[    4.926746] usb 1-1: new high-speed USB device number 2 using xhci_hcd
[/sbin/fsck.ext4 (1) -- /dev/mmcblk1p2] fsck.ext4 -y -C0 /dev/mmcblk1p2
e2fsck 1.46.2 (28-Feb-2021)
rootfs: recovering journal
Setting free inodes count to 61061 (was 61062)
Setting free blocks count to 87941 (was 89990)
rootfs: clean, 45019/106080 files, 335995/423936 blocks
done.
[    5.089325] usb 1-1: New USB device found, idVendor=2109, idProduct=3431, bcdDevice= 4.21
[    5.097723] usb 1-1: New USB device strings: Mfr=0, Product=1, SerialNumber=0
[    5.105016] usb 1-1: Product: USB2.0 Hub
[    5.107624] EXT4-fs (mmcblk1p2): mounted filesystem with ordered data mode. Opts: (null). Quota mode: none.
[    5.110685] hub 1-1:1.0: USB hub found
done.[    5.123104] hub 1-1:1.0: 4 ports detected

Begin: Running /scripts/local-bottom ... done.
Begin: Running /scripts/init-bottom ... done.
[    5.385800] Not activating Mandatory Access Control as /sbin/tomoyo-init does not exist.
[    5.471187] "init" (1) uses deprecated CP15 Barrier instruction at 0xf7a62310
[    5.520172] "init" (1) uses deprecated CP15 Barrier instruction at 0xf7a62310
[    5.527507] "init" (1) uses deprecated CP15 Barrier instruction at 0xf7a62310
[    5.534787] "init" (1) uses deprecated CP15 Barrier instruction at 0xf7a62310
[    5.548766] "init" (1) uses deprecated CP15 Barrier instruction at 0xf7a62310
[    5.556074] "init" (1) uses deprecated CP15 Barrier instruction at 0xf7a62310
[    5.563357] "init" (1) uses deprecated CP15 Barrier instruction at 0xf7a62310
[    5.574207] "init" (1) uses deprecated CP15 Barrier instruction at 0xf7a62310
[    5.587619] "init" (1) uses deprecated CP15 Barrier instruction at 0xf7a62310
[    5.594924] "init" (1) uses deprecated CP15 Barrier instruction at 0xf7a62310
[    5.742439] systemd[1]: System time before build time, advancing clock.
[    5.766409] systemd[1]: Failed to look up module alias 'autofs4': Function not implemented
[    5.815090] systemd[1]: systemd 247.3-6+rpi1 running in system mode. (+PAM +AUDIT +SELINUX +IMA +APPARMOR +SMACK +SYSVINIT +UTMP +LIBCRYPTSETUP +GCRYPT +GNUTLS +ACL +XZ +LZ4 +ZSTD +SECCOMP +BLKID +ELFUTILS +KMOD +IDN2 -IDN +PCRE2 default-hierarchy=unified)
[    5.839217] systemd[1]: Detected architecture arm64.

Welcome to Raspbian GNU/Linux 11 (bullseye)!

[    5.869871] systemd[1]: Set hostname to <raspberrypi>.
[    6.629272] systemd[1]: Queued start job for default target Graphical Interface.
[    6.638341] random: systemd: uninitialized urandom read (16 bytes read)
[    6.649497] systemd[1]: Created slice system-getty.slice.
[  OK  ] Created slice system-getty.slice.
[    6.670995] random: systemd: uninitialized urandom read (16 bytes read)
[    6.679520] systemd[1]: Created slice system-modprobe.slice.
[  OK  ] Created slice system-modprobe.slice.
[    6.702920] random: systemd: uninitialized urandom read (16 bytes read)
[    6.711776] systemd[1]: Created slice system-serial\x2dgetty.slice.
[  OK  ] Created slice system-serial\x2dgetty.slice.
[    6.736554] systemd[1]: Created slice system-systemd\x2dfsck.slice.
[  OK  ] Created slice system-systemd\x2dfsck.slice.
[    6.760276] systemd[1]: Created slice User and Session Slice.
[  OK  ] Created slice User and Session Slice.
[    6.783412] systemd[1]: Started Dispatch Password Requests to Console Directory Watch.
[  OK  ] Started Dispatch Password …ts to Console Directory Watch.
[    6.811196] systemd[1]: Started Forward Password Requests to Wall Directory Watch.
[  OK  ] Started Forward Password R…uests to Wall Directory Watch.
[    6.839225] systemd[1]: Starting of Arbitrary Executable File Formats File System Automount Point not supported.
[UNSUPP] Starting of Arbitrary Exec…Automount Point not supported.
[    6.867126] systemd[1]: Reached target Local Encrypted Volumes.
[  OK  ] Reached target Local Encrypted Volumes.
[    6.891184] systemd[1]: Reached target Paths.
[  OK  ] Reached target Paths.
[    6.911028] systemd[1]: Reached target Slices.
[  OK  ] Reached target Slices.
[    6.930997] systemd[1]: Reached target Swap.
[  OK  ] Reached target Swap.
[    6.951902] systemd[1]: Listening on Syslog Socket.
[  OK  ] Listening on Syslog Socket.
[    6.971525] systemd[1]: Listening on fsck to fsckd communication Socket.
[  OK  ] Listening on fsck to fsckd communication Socket.
[    6.995242] systemd[1]: Listening on initctl Compatibility Named Pipe.
[  OK  ] Listening on initctl Compatibility Named Pipe.
[    7.019987] systemd[1]: Listening on Journal Audit Socket.
[  OK  ] Listening on Journal Audit Socket.
[    7.043593] systemd[1]: Listening on Journal Socket (/dev/log).
[  OK  ] Listening on Journal Socket (/dev/log).
[    7.067650] systemd[1]: Listening on Journal Socket.
[  OK  ] Listening on Journal Socket.
[    7.094790] systemd[1]: Listening on udev Control Socket.
[  OK  ] Listening on udev Control Socket.
[    7.115637] systemd[1]: Listening on udev Kernel Socket.
[  OK  ] Listening on udev Kernel Socket.
[    7.144083] systemd[1]: Mounting Huge Pages File System...
         Mounting Huge Pages File System...
[    7.172042] systemd[1]: Mounting POSIX Message Queue File System...
         Mounting POSIX Message Queue File System...
[    7.199453] systemd[1]: Mounting RPC Pipe File System...
         Mounting RPC Pipe File System...
[    7.224660] systemd[1]: Mounting Kernel Debug File System...
         Mounting Kernel Debug File System...
[    7.251960] systemd[1]: Mounting Kernel Trace File System...
         Mounting Kernel Trace File System...
[    7.275081] systemd[1]: Condition check resulted in Kernel Module supporting RPCSEC_GSS being skipped.
[    7.292913] systemd[1]: Starting Restore / save the current clock...
         Starting Restore / save the current clock...
[    7.320299] systemd[1]: Starting Set the console keyboard layout...
         Starting Set the console keyboard layout...
[    7.343042] systemd[1]: Condition check resulted in Create list of static device nodes for the current kernel being skipped.
[    7.360235] systemd[1]: Starting Load Kernel Module configfs...
         Starting Load Kernel Module configfs...
[    7.388678] systemd[1]: Starting Load Kernel Module drm...
         Starting Load Kernel Module drm...
[    7.416456] systemd[1]: Starting Load Kernel Module fuse...
         Starting Load Kernel Module fuse...
[    7.443692] systemd[1]: Condition check resulted in Set Up Additional Binary Formats being skipped.
[    7.453288] systemd[1]: Condition check resulted in File System Check on Root Device being skipped.
[    7.471217] systemd[1]: Starting Journal Service...
         Starting Journal Service...
[    7.500989] systemd[1]: Starting Load Kernel Modules...
         Starting Load Kernel Modules...
[    7.528723] systemd[1]: Starting Remount Root and Kernel File Systems...
         Starting Remount Root and Kernel File Systems...
[    7.556893] systemd[1]: Starting Coldplug All udev Devices...
         Starting Coldplug All udev Devices...
[    7.592207] EXT4-fs (mmcblk1p2): re-mounted. Opts: (null). Quota mode: none.
[    7.595126] systemd[1]: Mounted Huge Pages File System.
[  OK  ] Mounted Huge Pages File System.
[    7.628863] systemd[1]: Mounted POSIX Message Queue File System.
[  OK  ] Mounted POSIX Message Queue File System.
[    7.651738] systemd[1]: run-rpc_pipefs.mount: Mount process exited, code=exited, status=32/n/a
[    7.660673] systemd[1]: run-rpc_pipefs.mount: Failed with result 'exit-code'.
[    7.670210] systemd[1]: Failed to mount RPC Pipe File System.
[FAILED] Failed to mount RPC Pipe File System.
See 'systemctl status run-rpc_pipefs.mount' for details.
[    7.714917] systemd[1]: Dependency failed for RPC security service for NFS client and server.
[DEPEND] Dependency failed for RPC …ice for NFS client and server.
[    7.746951] systemd[1]: rpc-gssd.service: Job rpc-gssd.service/start failed with result 'dependency'.
[    7.756462] systemd[1]: Dependency failed for RPC security service for NFS server.
[DEPEND] Dependency failed for RPC …curity service for NFS server.
[    7.782895] systemd[1]: rpc-svcgssd.service: Job rpc-svcgssd.service/start failed with result 'dependency'.
[    7.794026] systemd[1]: Started Journal Service.
[  OK  ] Started Journal Service.
[  OK  ] Mounted Kernel Debug File System.
[  OK  ] Mounted Kernel Trace File System.
[  OK  ] Finished Restore / save the current clock.
[  OK  ] Finished Set the console keyboard layout.
[  OK  ] Finished Load Kernel Module configfs.
[  OK  ] Finished Load Kernel Module drm.
[  OK  ] Finished Load Kernel Module fuse.
[  OK  ] Finished Load Kernel Modules.
[  OK  ] Finished Remount Root and Kernel File Systems.
[  OK  ] Reached target NFS client services.
[  OK  ] Reached target Remote File Systems (Pre).
[  OK  ] Reached target Remote File Systems.
         Starting Flush Journal to Persistent Storage...
         Starting Load/Save Random Seed...
[    8.095588] systemd-journald[243]: Received client request to flush runtime journal.
         Starting Apply Kernel Variables...
         Starting Create System Users...
[    8.133810] systemd-journald[243]: File /var/log/journal/25b2d71e31b444999950d1e057c44be9/system.journal corrupted or uncleanly shut down, renaming and replacing.
[  OK  ] Finished Apply Kernel Variables.
[  OK  ] Finished Create System Users.
         Starting Create Static Device Nodes in /dev...
[  OK  ] Finished Coldplug All udev Devices.
[  OK  ] Finished Flush Journal to Persistent Storage.
[  OK  ] Finished Create Static Device Nodes in /dev.
[  OK  ] Reached target Local File Systems (Pre).
         Starting Helper to synchronize boot up for ifupdown...
         Starting Rule-based Manage…for Device Events and Files...
[  OK  ] Finished Helper to synchronize boot up for ifupdown.
[  OK  ] Started Rule-based Manager for Device Events and Files.
[  OK  ] Found device /dev/ttyS1.
[  OK  ] Reached target Hardware activated USB gadget.
[  OK  ] Found device /dev/disk/by-partuuid/94319aae-01.
         Starting File System Check…isk/by-partuuid/94319aae-01...
[  OK  ] Started File System Check Daemon to report status.
[  OK  ] Finished File System Check…/disk/by-partuuid/94319aae-01.
         Mounting /boot...
[FAILED] Failed to mount /boot.
See 'systemctl status boot.mount' for details.
[DEPEND] Dependency failed for Local File Systems.
[  OK  ] Stopped Dispatch Password …ts to Console Directory Watch.
[  OK  ] Stopped Forward Password R…uests to Wall Directory Watch.
[  OK  ] Reached target Timers.
         Starting Set console font and keymap...
         Starting Raise network interfaces...
         Starting Preprocess NFS configuration...
[  OK  ] Closed Syslog Socket.
[  OK  ] Reached target Login Prompts.
[  OK  ] Reached target Sockets.
[  OK  ] Started Emergency Shell.
[  OK  ] Reached target Emergency Mode.
         Starting Create Volatile Files and Directories...
[  OK  ] Finished Set console font and keymap.
[  OK  ] Finished Preprocess NFS configuration.
[  OK  ] Finished Create Volatile Files and Directories.
         Starting Network Time Synchronization...
         Starting Update UTMP about System Boot/Shutdown...
[  OK  ] Finished Raise network interfaces.
[  OK  ] Reached target Network.
[  OK  ] Finished Update UTMP about System Boot/Shutdown.
[  OK  ] Finished Load/Save Random Seed.
         Starting Update UTMP about System Runlevel Changes...
[  OK  ] Finished Update UTMP about System Runlevel Changes.
[  OK  ] Started Network Time Synchronization.
[  OK  ] Reached target System Time Set.
[  OK  ] Reached target System Time Synchronized.
You are in emergency mode. After logging in, type "journalctl -xb" to view
system logs, "systemctl reboot" to reboot, "systemctl defaul
Cannot open access to console, the root account is locked.
See sulogin(8) man page for more details.

Press Enter to continue.

```