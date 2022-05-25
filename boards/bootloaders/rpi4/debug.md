```powershell
rustBoot3 on  main [✘?] via 🦀 v1.62.0-nightly
❯ terminal-s.exe
--- COM3 is connected. Press Ctrl+] to quit ---
[    1.923912] EMMC: reset card.
[    1.923941] control1: 16143
[    1.926617] Divisor = 63, Freq Set = 396825
[    2.333993] CSD Contents : 00 40 0e 00 32 5b 59 00 00ed c8 7f 80 0a 40 40
[    2.337810] cemmc_structure=1, spec_vers=0, taac=0x0E, nsac=0x00, tran_speed=0x32,ccc=0x05B5, read_bl_len=0x09, read_bl_partial=0b, write_blk_misalign=0b,read_blk_misalign=0b, dsr_imp=0b, sector_size =0x7F, erase_blk_en=1b
[    2.357443] CSD 2.0: ver2_c_size = 0xEFFC, card capacity: 31914459136 bytes or 31.91GiB
[    2.365349] wp_grp_size=0x0000000b, wp_grp_enable=0b, default_ecc=00b, r2w_factor=010b, write_bl_len=0x09, write_bl_partial=0b, file_format_grp=0, copy=1b, perm_write_protect=0b, tmp_write_protect=0b, file_format=0b ecc=00b
[    2.385069] control1: 271
[    2.387589] Divisor = 1, Freq Set = 25000000
[    2.394074] EMMC: Bus width set to 4
[    2.395236] EMMC: SD Card Type 2 HC, 30436Mb, mfr_id: 3, 'SD:ACLCD', r8.0, mfr_date: 1/2017, serial: 0xbbce119c, RCA: 0xaaaa
[    2.406353] EMMC2 driver initialized...

[    2.410176] rpi4 version 0.1.0
[    2.413130] Booting on: Raspberry Pi 4
[    2.416778] MMU online. Special regions:
[    2.420601]       0x00080000 - 0x000a3fff | 144 KiB | C   RO PX  | Kernel code and RO data
[    2.428767]       0xfe000000 - 0xff84ffff |  24 MiB | Dev RW PXN | Device MMIO
[    2.435891] Current privilege level: EL1
[    2.439713] Exception handling state:
[    2.443275]       Debug:  Masked
[    2.446403]       SError: Masked
[    2.449530]       IRQ:    Masked
[    2.452658]       FIQ:    Masked
[    2.455785] Architectural timer resolution: 18 ns
[    2.460390] Drivers loaded:
[    2.463083]       1. BCM GPIO
[    2.465950]       2. BCM PL011 UART
[    2.469338] Chars written: 1702
[    2.618146] fat cache populated ...
[    2.618662] Listing root directory:
[    2.623059]      - Found: SIGNED~1.ITB
[    2.627486] loading fit-image...
[    6.052745] loaded fit: 62202019 bytes, starting at addr: 0x4200000
[    6.056041] authenticating fit-image...
######## ecdsa signature checks out, image is authentic ########
[    8.441000] relocating kernel to addr: 0x2200000
[    8.464749] relocating initrd to addr: 0x200000
[    8.466307] load rbconfig...
[    8.469121] patching dtb...
[    8.472974] relocating dtb to addr: 0x4000000

*************** Starting kernel ***************

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
[    0.000219] Console: colour dummy device 80x25
[    0.000587] printk: console [tty0] enabled
[    0.000686] Calibrating delay loop (skipped), value calculated using timer frequency.. 108.00 BogoMIPS (lpj=216000)
[    0.000716] pid_max: default: 32768 minimum: 301
[    0.000864] LSM: Security Framework initializing
[    0.000904] Yama: disabled by default; enable with sysctl kernel.yama.*
[    0.001054] AppArmor: AppArmor initialized
[    0.001075] TOMOYO Linux initialized
[    0.001213] Mount-cache hash table entries: 8192 (order: 4, 65536 bytes, linear)
[    0.001293] Mountpoint-cache hash table entries: 8192 (order: 4, 65536 bytes, linear)
[    0.003780] rcu: Hierarchical SRCU implementation.
[    0.005998] EFI services will not be available.
[    0.006560] smp: Bringing up secondary CPUs ...
[    0.007241] Detected PIPT I-cache on CPU1
[    0.007316] CPU1: Booted secondary processor 0x0000000001 [0x410fd083]
[    0.008158] Detected PIPT I-cache on CPU2
[    0.008204] CPU2: Booted secondary processor 0x0000000002 [0x410fd083]
[    0.009022] Detected PIPT I-cache on CPU3
[    0.009070] CPU3: Booted secondary processor 0x0000000003 [0x410fd083]
[    0.009180] smp: Brought up 1 node, 4 CPUs
[    0.009248] SMP: Total of 4 processors activated.
[    0.009262] CPU features: detected: 32-bit EL0 Support
[    0.009273] CPU features: detected: 32-bit EL1 Support
[    0.009287] CPU features: detected: CRC32 instructions
[    0.027635] ------------[ cut here ]------------
[    0.027670] CPU: CPUs started in inconsistent modes
[    0.027683] WARNING: CPU: 0 PID: 1 at arch/arm64/kernel/smp.c:426 smp_cpus_done+0x78/0xc4
[    0.027737] Modules linked in:
[    0.027754] CPU: 0 PID: 1 Comm: swapper/0 Not tainted 5.15.0-trunk-arm64 #1  Debian 5.15.1-1~exp1+apertis1
[    0.027779] Hardware name: Raspberry Pi 4 Model B (DT)
[    0.027792] pstate: 60000005 (nZCv daif -PAN -UAO -TCO -DIT -SSBS BTYPE=--)
[    0.027812] pc : smp_cpus_done+0x78/0xc4
[    0.027832] lr : smp_cpus_done+0x78/0xc4
[    0.027850] sp : ffff800011cabe00
[    0.027861] x29: ffff800011cabe00 x28: 0000000000000000 x27: 0000000000000000
[    0.027890] x26: 0000000000000000 x25: 0000000000000000 x24: 0000000000000000
[    0.027916] x23: 0000000000000000 x22: ffff800011bf1000 x21: 0000000000000000
[    0.027941] x20: ffff800011527e40 x19: ffff800011a20000 x18: 0000000000000001
[    0.027966] x17: 0000000093e73ce6 x16: 000000008553e912 x15: 0720072007200720
[    0.027992] x14: 0720072d072d072d x13: 7365646f6d20746e x12: 65747369736e6f63
[    0.028017] x11: ffff800011a00288 x10: ffff8000119a9588 x9 : ffff80001011cc1c
[    0.028041] x8 : ffff8000119a7b68 x7 : ffff8000119ffb68 x6 : fffffffffffe1418
[    0.028065] x5 : 0000000000001a20 x4 : 000000000000aff5 x3 : 0000000000000000
[    0.028089] x2 : 0000000000000000 x1 : 0000000000000000 x0 : ffff0000401b9e80
[    0.028114] Call trace:
[    0.028126]  smp_cpus_done+0x78/0xc4
[    0.028146]  smp_init+0x88/0x98
[    0.028167]  kernel_init_freeable+0x194/0x328
[    0.028188]  kernel_init+0x30/0x140
[    0.028205]  ret_from_fork+0x10/0x20
[    0.028230] ---[ end trace 8b61479c8fdff1ef ]---
[    0.028337] alternatives: patching kernel code
[    0.118260] node 0 deferred pages initialised in 88ms
[    0.119882] devtmpfs: initialized
[    0.128254] Registered cp15_barrier emulation handler
[    0.128305] Registered setend emulation handler
[    0.128323] KASLR disabled due to lack of seed
[    0.128627] clocksource: jiffies: mask: 0xffffffff max_cycles: 0xffffffff, max_idle_ns: 7645041785100000 ns
[    0.128722] futex hash table entries: 1024 (order: 4, 65536 bytes, linear)
[    0.129273] pinctrl core: initialized pinctrl subsystem
[    0.129997] DMI not present or invalid.
[    0.130628] NET: Registered PF_NETLINK/PF_ROUTE protocol family
[    0.135663] DMA: preallocated 512 KiB GFP_KERNEL pool for atomic allocations
[    0.136538] DMA: preallocated 512 KiB GFP_KERNEL|GFP_DMA pool for atomic allocations
[    0.137697] DMA: preallocated 512 KiB GFP_KERNEL|GFP_DMA32 pool for atomic allocations
[    0.137834] audit: initializing netlink subsys (disabled)
[    0.138158] audit: type=2000 audit(0.136:1): state=initialized audit_enabled=0 res=1
[    0.139357] thermal_sys: Registered thermal governor 'fair_share'
[    0.139366] thermal_sys: Registered thermal governor 'bang_bang'
[    0.139385] thermal_sys: Registered thermal governor 'step_wise'
[    0.139398] thermal_sys: Registered thermal governor 'user_space'
[    0.139410] thermal_sys: Registered thermal governor 'power_allocator'
[    0.139626] cpuidle: using governor ladder
[    0.139673] cpuidle: using governor menu
[    0.139831] hw-breakpoint: found 6 breakpoint and 4 watchpoint registers.
[    0.139971] ASID allocator initialised with 65536 entries
[    0.140399] Serial: AMBA PL011 UART driver
[    0.167118] HugeTLB registered 1.00 GiB page size, pre-allocated 0 pages
[    0.167158] HugeTLB registered 32.0 MiB page size, pre-allocated 0 pages
[    0.167175] HugeTLB registered 2.00 MiB page size, pre-allocated 0 pages
[    0.167190] HugeTLB registered 64.0 KiB page size, pre-allocated 0 pages
[    0.826342] ACPI: Interpreter disabled.
[    0.826979] iommu: Default domain type: Translated
[    0.826999] iommu: DMA domain TLB invalidation policy: strict mode
[    0.827294] vgaarb: loaded
[    0.827729] EDAC MC: Ver: 3.0.0
[    0.829463] NetLabel: Initializing
[    0.829484] NetLabel:  domain hash size = 128
[    0.829496] NetLabel:  protocols = UNLABELED CIPSOv4 CALIPSO
[    0.829584] NetLabel:  unlabeled traffic allowed by default
[    0.830013] clocksource: Switched to clocksource arch_sys_counter
[    0.875371] VFS: Disk quotas dquot_6.6.0
[    0.875476] VFS: Dquot-cache hash table entries: 512 (order 0, 4096 bytes)
[    0.876361] AppArmor: AppArmor Filesystem Enabled
[    0.876591] pnp: PnP ACPI: disabled
[    0.886037] NET: Registered PF_INET protocol family
[    0.886613] IP idents hash table entries: 65536 (order: 7, 524288 bytes, linear)
[    0.889724] tcp_listen_portaddr_hash hash table entries: 2048 (order: 3, 32768 bytes, linear)
[    0.890066] TCP established hash table entries: 32768 (order: 6, 262144 bytes, linear)
[    0.890778] TCP bind hash table entries: 32768 (order: 7, 524288 bytes, linear)
[    0.891083] TCP: Hash tables configured (established 32768 bind 32768)
[    0.891573] MPTCP token hash table entries: 4096 (order: 4, 98304 bytes, linear)
[    0.891780] UDP hash table entries: 2048 (order: 4, 65536 bytes, linear)
[    0.891909] UDP-Lite hash table entries: 2048 (order: 4, 65536 bytes, linear)
[    0.892583] NET: Registered PF_UNIX/PF_LOCAL protocol family
[    0.892630] NET: Registered PF_XDP protocol family
[    0.892653] PCI: CLS 0 bytes, default 64
[    0.893341] Trying to unpack rootfs image as initramfs...
[    0.898800] hw perfevents: enabled with armv8_cortex_a72 PMU driver, 7 counters available
[    0.899015] kvm [1]: HYP mode not available
[    0.900386] Initialise system trusted keyrings
[    0.900446] Key type blacklist registered
[    0.900698] workingset: timestamp_bits=42 max_order=20 bucket_order=0
[    0.907178] zbud: loaded
[    0.908079] integrity: Platform Keyring initialized
[    0.908107] Key type asymmetric registered
[    0.908121] Asymmetric key parser 'x509' registered
[    0.908253] Block layer SCSI generic (bsg) driver version 0.4 loaded (major 248)
[    0.908546] io scheduler mq-deadline registered
[    0.917105] shpchp: Standard Hot Plug PCI Controller Driver version: 0.4
[    0.918066] brcm-pcie fd500000.pcie: host bridge /scb/pcie@7d500000 ranges:
[    0.918104] brcm-pcie fd500000.pcie:   No bus range found for /scb/pcie@7d500000, using [bus 00-ff]
[    0.918153] brcm-pcie fd500000.pcie:      MEM 0x0600000000..0x0603ffffff -> 0x00f8000000
[    0.918205] brcm-pcie fd500000.pcie:   IB MEM 0x0000000000..0x00bfffffff -> 0x0000000000
[    0.984104] brcm-pcie fd500000.pcie: link up, 5.0 GT/s PCIe x1 (SSC)
[    0.984427] brcm-pcie fd500000.pcie: PCI host bridge to bus 0000:00
[    0.984451] pci_bus 0000:00: root bus resource [bus 00-ff]
[    0.984473] pci_bus 0000:00: root bus resource [mem 0x600000000-0x603ffffff] (bus address [0xf8000000-0xfbffffff])
[    0.984533] pci 0000:00:00.0: [14e4:2711] type 01 class 0x060400
[    0.984632] pci 0000:00:00.0: PME# supported from D0 D3hot
[    0.987217] pci 0000:01:00.0: [1106:3483] type 00 class 0x0c0330
[    0.987305] pci 0000:01:00.0: reg 0x10: [mem 0x00000000-0x00000fff 64bit]
[    0.987502] pci 0000:01:00.0: PME# supported from D0 D3hot
[    1.000094] pci 0000:00:00.0: BAR 14: assigned [mem 0x600000000-0x6000fffff]
[    1.000141] pci 0000:01:00.0: BAR 0: assigned [mem 0x600000000-0x600000fff 64bit]
[    1.000174] pci 0000:00:00.0: PCI bridge to [bus 01]
[    1.000191] pci 0000:00:00.0:   bridge window [mem 0x600000000-0x6000fffff]
[    1.000435] pcieport 0000:00:00.0: enabling device (0000 -> 0002)
[    1.000622] pcieport 0000:00:00.0: PME: Signaling with IRQ 50
[    1.000996] pcieport 0000:00:00.0: AER: enabled with IRQ 50
[    1.012257] Serial: 8250/16550 driver, 4 ports, IRQ sharing enabled
[    1.014426] fe215040.serial: ttyS1 at MMIO 0xfe215040 (irq = 26, base_baud = 24999999) is a 16550
[    2.161429] printk: console [ttyS1] enabled
[    2.166874] Serial: AMBA driver
[    2.170135] SuperH (H)SCI(F) driver initialized
[    2.175356] msm_serial: driver initialized
[    2.180600] cacheinfo: Unable to detect cache hierarchy for CPU 0
[    2.187730] bcm2835-power bcm2835-power: Broadcom BCM2835 power domains driver
[    2.196121] mousedev: PS/2 mouse device common for all mice
[    2.203068] brcmstb-i2c fef04500.i2c:  @97500hz registered in polling mode
[    2.210419] brcmstb-i2c fef09500.i2c:  @97500hz registered in polling mode
[    2.218943] ledtrig-cpu: registered to indicate activity on CPUs
[    2.226030] bcm2835-mbox fe00b880.mailbox: mailbox enabled
[    2.233803] NET: Registered PF_INET6 protocol family
[    3.333063] Freeing initrd memory: 32128K
[    3.375231] Segment Routing with IPv6
[    3.379076] In-situ OAM (IOAM) with IPv6
[    3.383193] mip6: Mobile IPv6
[    3.386230] NET: Registered PF_PACKET protocol family
[    3.391542] mpls_gso: MPLS GSO support
[    3.396163] registered taskstats version 1
[    3.400365] Loading compiled-in X.509 certificates
[    3.555474] Loaded X.509 cert 'Debian Secure Boot CA: 6ccece7e4c6c0d1f6149f3dd27dfcc5cbb419ea1'
[    3.564419] Loaded X.509 cert 'Debian Secure Boot Signer 2021 - linux: 4b6ef5abca669825178e052c84667ccbc0531f8c'
[    3.575780] zswap: loaded using pool lzo/zbud
[    3.581081] Key type ._fscrypt registered
[    3.585195] Key type .fscrypt registered
[    3.589196] Key type fscrypt-provisioning registered
[    3.611647] Key type encrypted registered
[    3.615778] AppArmor: AppArmor sha1 policy hashing enabled
[    3.621404] ima: No TPM chip found, activating TPM-bypass!
[    3.627018] ima: Allocated hash algorithm: sha256
[    3.631862] ima: No architecture policies found
[    3.636533] evm: Initialising EVM extended attributes:
[    3.641768] evm: security.selinux
[    3.645146] evm: security.SMACK64 (disabled)
[    3.649494] evm: security.SMACK64EXEC (disabled)
[    3.654194] evm: security.SMACK64TRANSMUTE (disabled)
[    3.659336] evm: security.SMACK64MMAP (disabled)
[    3.664035] evm: security.apparmor
[    3.667498] evm: security.ima
[    3.670520] evm: security.capability
[    3.674159] evm: HMAC attrs: 0x1
[    3.683043] fe201000.serial: ttyAMA0 at MMIO 0xfe201000 (irq = 24, base_baud = 0) is a PL011 rev2
[    3.692381] serial serial0: tty port ttyAMA0 registered
[    3.698855] raspberrypi-firmware soc:firmware: Attached to firmware from 2021-04-30T13:45:52
[    3.882246] Freeing unused kernel memory: 5760K
[    3.944149] Checked W+X mappings: passed, no W+X pages found
[    3.949969] Run /init as init process
Loading, please wait...
Starting version 247.3-6+apertis1bv2022dev3b2
[    4.316109] phy_generic: module verification failed: signature and/or required key missing - tainting kernel
[    4.405590] sdhci: Secure Digital Host Controller Interface driver
[    4.411962] sdhci: Copyright(c) Pierre Ossman
[    4.418516] sdhci-pltfm: SDHCI platform and OF driver helper
[    4.431287] usb_phy_generic phy: supply vcc not found, using dummy regulator
[    4.434811] libphy: Fixed MDIO Bus: probed
[    4.439688] sdhci-iproc fe300000.mmc: allocated mmc-pwrseq
[    4.446497] usbcore: registered new interface driver usbfs
[    4.455359] usbcore: registered new interface driver hub
[    4.463099] usbcore: registered new device driver usb
[    4.469635] bcmgenet fd580000.ethernet: GENET 5.0 EPHY: 0x0000
[    4.475689] bcmgenet fd580000.ethernet: using random Ethernet MAC
[    4.494088] mmc0: SDHCI controller on fe300000.mmc [fe300000.mmc] using PIO
[    4.494094] libphy: bcmgenet MII bus: probed
[    4.513629] mmc1: SDHCI controller on fe340000.mmc [fe340000.mmc] using ADMA
[    4.516405] dwc2 fe980000.usb: supply vusb_d not found, using dummy regulator
[    4.530758] dwc2 fe980000.usb: supply vusb_a not found, using dummy regulator
[    4.568195] xhci_hcd 0000:01:00.0: xHCI Host Controller
[    4.570195] unimac-mdio unimac-mdio.-19: Broadcom UniMAC MDIO bus
[    4.573638] xhci_hcd 0000:01:00.0: new USB bus registered, assigned bus number 1
[    4.588998] xhci_hcd 0000:01:00.0: hcc params 0x002841eb hci version 0x100 quirks 0x0000040000000890
[    4.599579] usb usb1: New USB device found, idVendor=1d6b, idProduct=0002, bcdDevice= 5.15
[    4.608095] usb usb1: New USB device strings: Mfr=3, Product=2, SerialNumber=1
[    4.615499] usb usb1: Product: xHCI Host Controller
[    4.620506] usb usb1: Manufacturer: Linux 5.15.0-trunk-arm64 xhci-hcd
[    4.627101] usb usb1: SerialNumber: 0000:01:00.0
[    4.632714] hub 1-0:1.0: USB hub found
[    4.636721] hub 1-0:1.0: 1 port detected
[    4.641628] xhci_hcd 0000:01:00.0: xHCI Host Controller
[    4.642439] dwc2 fe980000.usb: EPs: 8, dedicated fifos, 4080 entries in SPRAM
[    4.647096] xhci_hcd 0000:01:00.0: new USB bus registered, assigned bus number 2
[    4.661909] xhci_hcd 0000:01:00.0: Host supports USB 3.0 SuperSpeed
[    4.668776] usb usb2: New USB device found, idVendor=1d6b, idProduct=0003, bcdDevice= 5.15
[    4.677260] usb usb2: New USB device strings: Mfr=3, Product=2, SerialNumber=1
[    4.684666] usb usb2: Product: xHCI Host Controller
[    4.689687] usb usb2: Manufacturer: Linux 5.15.0-trunk-arm64 xhci-hcd
[    4.696287] usb usb2: SerialNumber: 0000:01:00.0
[    4.701766] hub 2-0:1.0: USB hub found
[    4.705721] hub 2-0:1.0: 4 ports detected
[    4.715466] random: fast init done
[    4.745348] mmc1: new ultra high speed DDR50 SDHC card at address aaaa
Begin: Loading essential[    4.753505] mmcblk1: mmc1:aaaa ACLCD 29.7 GiB
 drivers ... [    4.765455]  mmcblk1: p1 p2
[    4.779227] mmc0: new high speed SDIO card at address 0001
done.
Begin: Running /scripts/init-premount ... done.
Begin: Mounting root file system ... Begin: Running /scripts/local-top ... done.
Begin: Running /scripts/local-premount ... done.
[    4.902044] usb 1-1: new high-speed USB device number 2 using xhci_hcd
Begin: Waiting for root file system ... [    5.056662] usb 1-1: New USB device found, idVendor=2109, idProduct=3431, bcdDevice= 4.21
[    5.065039] usb 1-1: New USB device strings: Mfr=0, Product=1, SerialNumber=0
[    5.072313] usb 1-1: Product: USB2.0 Hub
[    5.079015] hub 1-1:1.0: USB hub found
[    5.083063] hub 1-1:1.0: 4 ports detected
Begin: Running /scripts/local-block ... done.
Begin: Running /scripts/local-block ... done.
Begin: Running /scripts/local-block ... done.
Begin: Running /scripts/local-block ... done.
Begin: Running /scripts/local-block ... done.
Begin: Running /scripts/local-block ... done.
Begin: Running /scripts/local-block ... done.
Begin: Running /scripts/local-block ... done.
Begin: Running /scripts/local-block ... done.
Begin: Running /scripts/local-block ... done.
Begin: Running /scripts/local-block ... done.
Begin: Running /scripts/local-block ... done.
Begin: Running /scripts/local-block ... done.
Begin: Running /scripts/local-block ... done.
Begin: Running /scripts/local-block ... done.
Begin: Running /scripts/local-block ... done.
Begin: Running /scripts/local-block ... done.
Begin: Running /scripts/local-block ... done.
Begin: Running /scripts/local-block ... done.
Begin: Running /scripts/local-block ... done.
Begin: Running /scripts/local-block ... done.
Begin: Running /scripts/local-block ... done.
Begin: Running /scripts/local-block ... done.
Begin: Running /scripts/local-block ... done.
Begin: Running /scripts/local-block ... done.
Begin: Running /scripts/local-block ... done.
Begin: Running /scripts/local-block ... done.
Begin: Running /scripts/local-block ... done.
Begin: Running /scripts/local-block ... done.
done.
Gave up waiting for root file system device.  Common problems:
 - Boot args (cat /proc/cmdline)
   - Check rootdelay= (did the system wait long enough?)
 - Missing modules (cat /proc/modules; ls /dev)
ALERT!  UUID=64bc182a-ca9d-4aa1-8936-d2919863c22a does not exist.  Dropping to a shell!


BusyBox v1.30.1 (Apertis 1:1.30.1-6+apertis2bv2022dev3b1) built-in shell (ash)
Enter 'help' for a list of built-in commands.

(initramfs) ls
bin      dev      init     proc     run      scripts  tmp      var
conf     etc      lib      root     sbin     sys      usr
(initramfs)

```