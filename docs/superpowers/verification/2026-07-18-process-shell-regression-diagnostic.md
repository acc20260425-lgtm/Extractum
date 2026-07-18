# Process Shell Regression Diagnostic Verification

**Session:** `1d805fa7-86f8-48f4-90ef-b55ef13720b1`
**Outcome:** `environment_precision_insufficient`
**Protocol commit:** `783c46a1eacce8c92b5e73efbaed247ef57a99d6`
**Protocol-lock blob:** `dc46dde0b4b7e7702eba05af3113c100f2fb8799`
**Protocol-lock SHA-256:** `8c087325c7b6c639c503e35fd5e1f3b7c4006dc03f6e53032654d45603a36a49`
**Raw artifact directory:** `C:\Users\Dima\AppData\Local\Temp\extractum-process-shell-sessions\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1`
**Artifact-index SHA-256:** `b7d4ad9ddacf21bcf14f38f3efbe93aea525b8b9a0cf2202469b06699bdbd6fb` (2842 files, 129183825 bytes)
**Recorded attempt span:** 82.4 minutes

## Environment

| Field | Value |
| --- | --- |
| platform | win32 |
| architecture | x64 |
| host | x86_64-pc-windows-msvc |
| cargo | cargo 1.95.0 (f2d3ce0bd 2026-03-21) |
| rustc | rustc 1.95.0 (59807616e 2026-04-14) binary: rustc commit-hash: 59807616e1fa2540724bfbac14d7976d7e4a3860 commit-date: 2026-04-14 host: x86_64-pc-windows-msvc release: 1.95.0 LLVM version: 22.1.2 |
| node | v24.13.1 |
| power | GUID схемы питания: 381b4222-f694-41f0-9685-ff5bb260df2e  (Сбалансированная) |
| defender | {"RealTimeProtectionEnabled":true,"AntivirusEnabled":true,"QuickScanAge":0} |
| processQuiescence | [] |
| operatorProcessAttestation | true |
| cargoEnvironment | {"CARGO_BUILD_TARGET":null,"CARGO_ENCODED_RUSTFLAGS":null,"CARGO_INCREMENTAL":null,"CARGO_TARGET_DIR":null,"RUSTFLAGS":null} |
| mainRoot | G:\Develop\Extractum |
| mainSrcTauriTree | fd9711a041432ef420e7b09d56a46131a2a52a2a |
| mainTargetDirectory | G:\Develop\Extractum\src-tauri\target |
| mainTargetSnapshot | {"exists":true,"digest":"f1e6d09eab831f78aa59794cdbbb612d5012676d10c62f5917455695824e5439","records":22566} |

## Attempt ledger

| Attempt | Status | Reasons | Started | Ended |
| --- | --- | --- | --- | --- |
| attempt-001 | infrastructure_invalid | coordinator_failure | 2026-07-18T11:35:45.651Z | 2026-07-18T11:39:01.225Z |
| attempt-002 | stability_invalid | block_unstable:A0, block_unstable:B, block_unstable:C, block_unstable:A2, block_unstable:D, block_unstable:A3, anchor_range_exceeded | 2026-07-18T11:41:34.380Z | 2026-07-18T12:19:02.446Z |
| attempt-003 | stability_invalid | block_unstable:A0, block_unstable:B, block_unstable:A1, block_unstable:C, block_unstable:D, block_unstable:A3, anchor_range_exceeded | 2026-07-18T12:20:33.432Z | 2026-07-18T12:58:09.887Z |

## Attempt environments

| Attempt | Host | Power | Defender | Target |
| --- | --- | --- | --- | --- |
| attempt-001 | x86_64-pc-windows-msvc | GUID схемы питания: 381b4222-f694-41f0-9685-ff5bb260df2e  (Сбалансированная) | {"RealTimeProtectionEnabled":true,"AntivirusEnabled":true,"QuickScanAge":0} | G:\Develop\Extractum\.worktrees\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempt-001\src-tauri\target |
| attempt-002 | x86_64-pc-windows-msvc | GUID схемы питания: 381b4222-f694-41f0-9685-ff5bb260df2e  (Сбалансированная) | {"RealTimeProtectionEnabled":true,"AntivirusEnabled":true,"QuickScanAge":0} | G:\Develop\Extractum\.worktrees\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempt-002\src-tauri\target |
| attempt-003 | x86_64-pc-windows-msvc | GUID схемы питания: 381b4222-f694-41f0-9685-ff5bb260df2e  (Сбалансированная) | {"RealTimeProtectionEnabled":true,"AntivirusEnabled":true,"QuickScanAge":0} | G:\Develop\Extractum\.worktrees\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempt-003\src-tauri\target |

## Retry and invalidation audit

| Attempt | Invalidation | Unexplained stability | Corrected cause | Action | Count |
| --- | --- | --- | --- | --- | ---: |
| attempt-001 | infrastructure_invalid | false | Windows Git checkout failed with Filename too long in attempt-001; verified correction: process-local core.longpaths=true materialized and removed a clean detached worktree at a longer root without changing Git config. | retry | 0 |
| attempt-002 | stability_invalid | true | none | retry | 1 |
| attempt-003 | stability_invalid | true | none | environment_precision_insufficient | 2 |

### Attempt error details

| Attempt | Kind | Error kind | Category | Message |
| --- | --- | --- | --- | --- |
| attempt-001 | infrastructure_invalid | preflight_command_failed | none | attempt-001-worktree-add |
| attempt-002 | stability_invalid | none | none | none |
| attempt-003 | stability_invalid | none | none | none |

### Corrected environment deltas

| Attempt | Corrected cause | Before | After |
| --- | --- | --- | --- |
| none | none | none | none |

## Attempt attempt-001 raw measurements

**Recorded kind:** `infrastructure_invalid`

**Recalculated stability reasons:** none / infrastructure invalidation

| Block | Wall samples | Median | Within 300 ms | No-op wall | No-op Cargo |
| --- | --- | ---: | ---: | ---: | ---: |


### State evidence

| Block | src-tauri tree | Canonical lib.rs SHA-256 | Metadata direct edge | Cargo target |
| --- | --- | --- | --- | --- |


### Cargo-reported samples and diagnostics

| Block | Cargo durations (ms) | `--extern extractum_process` | Feature graph | Timings HTML | SHA-256 |
| --- | --- | --- | --- | --- | --- |


## Attempt attempt-002 raw measurements

**Recorded kind:** `stability_invalid`

**Recalculated stability reasons:** block_unstable:A0, block_unstable:B, block_unstable:C, block_unstable:A2, block_unstable:D, block_unstable:A3, anchor_range_exceeded

| Block | Wall samples | Median | Within 300 ms | No-op wall | No-op Cargo |
| --- | --- | ---: | ---: | ---: | ---: |
| A0 | 12436.5736, 9440.541, 10204.9321, 12666.8992, 10686.682, 10140.5635, 13276.4182 | 10686.682 ms | 1/7 | 1420.8449 ms | 1290 ms |
| B | 9464.0458, 11134.6597, 9544.065, 12150.7221, 10061.3042, 9640.5216, 9630.6574 | 9640.5216 ms | 4/7 | 1326.9793 ms | 1190 ms |
| A1 | 9470.0395, 13282.8013, 9472.5038, 9293.3702, 9801.9746, 11721.9857, 9502.6584 | 9502.6584 ms | 5/7 | 1304.9863 ms | 1190 ms |
| C | 11001.2702, 9576.8184, 16375.8938, 9831.8996, 9558.1717, 9779.8322, 11328.8088 | 9831.8996 ms | 4/7 | 1311.4154 ms | 1190 ms |
| A2 | 12709.891, 9697.381, 9424.1823, 9336.6221, 11213.745, 14680.9737, 10766.2067 | 10766.2067 ms | 1/7 | 1706.6503 ms | 1560 ms |
| D | 13475.2664, 9944.9592, 9406.8213, 11825.5652, 12633.4612, 10657.3666, 9227.943 | 10657.3666 ms | 1/7 | 1319.9202 ms | 1200 ms |
| A3 | 9343.4949, 10058.6442, 14909.5684, 9475.9878, 9814.567, 10307.7171, 11168.5851 | 10058.6442 ms | 3/7 | 1317.1707 ms | 1200 ms |

### State evidence

| Block | src-tauri tree | Canonical lib.rs SHA-256 | Metadata direct edge | Cargo target |
| --- | --- | --- | --- | --- |
| A0 | fd9711a041432ef420e7b09d56a46131a2a52a2a | ec21466993001f6d3c76bfc56edb3959478ddca0a3f86a42d278617f4ac89312 | false | G:\Develop\Extractum\.worktrees\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempt-002\src-tauri\target |
| B | 34ba61e94780ac68db1bcb38edc23dfbdcfa44a3 | ec21466993001f6d3c76bfc56edb3959478ddca0a3f86a42d278617f4ac89312 | false | G:\Develop\Extractum\.worktrees\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempt-002\src-tauri\target |
| A1 | fd9711a041432ef420e7b09d56a46131a2a52a2a | ec21466993001f6d3c76bfc56edb3959478ddca0a3f86a42d278617f4ac89312 | false | G:\Develop\Extractum\.worktrees\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempt-002\src-tauri\target |
| C | cb07b39d4f2b2598f76c495f5f0a8287006c2fbf | ec21466993001f6d3c76bfc56edb3959478ddca0a3f86a42d278617f4ac89312 | true | G:\Develop\Extractum\.worktrees\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempt-002\src-tauri\target |
| A2 | fd9711a041432ef420e7b09d56a46131a2a52a2a | ec21466993001f6d3c76bfc56edb3959478ddca0a3f86a42d278617f4ac89312 | false | G:\Develop\Extractum\.worktrees\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempt-002\src-tauri\target |
| D | 77e2d163ccc8bddf3ea051cb995909888cae9aba | c5f678eb13e83050b1a13e0fb34f4fc53310e847889cea551a256066e7c958ce | true | G:\Develop\Extractum\.worktrees\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempt-002\src-tauri\target |
| A3 | fd9711a041432ef420e7b09d56a46131a2a52a2a | ec21466993001f6d3c76bfc56edb3959478ddca0a3f86a42d278617f4ac89312 | false | G:\Develop\Extractum\.worktrees\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempt-002\src-tauri\target |

### Cargo-reported samples and diagnostics

| Block | Cargo durations (ms) | `--extern extractum_process` | Feature graph | Timings HTML | SHA-256 |
| --- | --- | --- | --- | --- | --- |
| A0 | 12300, 9310, 10080, 12520, 10550, 10000, 13150 | false | C:\Users\Dima\AppData\Local\Temp\extractum-process-shell-sessions\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempts\attempt-002\runs\A0.feature-tree.stdout.log | C:\Users\Dima\AppData\Local\Temp\extractum-process-shell-sessions\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempts\attempt-002\timings\A0.diagnostic.dirty.html | 28d22fe8fa46e16a14c29d36a959a3e0dda7175a45f880844dc66a06a208f82b |
| B | 9340, 11010, 9410, 12020, 9920, 9520, 9510 | false | C:\Users\Dima\AppData\Local\Temp\extractum-process-shell-sessions\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempts\attempt-002\runs\B.feature-tree.stdout.log | C:\Users\Dima\AppData\Local\Temp\extractum-process-shell-sessions\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempts\attempt-002\timings\B.diagnostic.dirty.html | c76ca4e8846ebf825167c6d286ed6abb2af4469565ea941482f2428d397aff43 |
| A1 | 9350, 13130, 9360, 9170, 9690, 11540, 9380 | false | C:\Users\Dima\AppData\Local\Temp\extractum-process-shell-sessions\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempts\attempt-002\runs\A1.feature-tree.stdout.log | C:\Users\Dima\AppData\Local\Temp\extractum-process-shell-sessions\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempts\attempt-002\timings\A1.diagnostic.dirty.html | edb179aea891cfbad0d87884ce6693bf5806e4515b35ff40f52f7fea5533e18c |
| C | 10860, 9450, 16230, 9700, 9430, 9660, 11190 | true | C:\Users\Dima\AppData\Local\Temp\extractum-process-shell-sessions\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempts\attempt-002\runs\C.feature-tree.stdout.log | C:\Users\Dima\AppData\Local\Temp\extractum-process-shell-sessions\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempts\attempt-002\timings\C.diagnostic.dirty.html | 7cd45316477bb8a08cd0061fde02b3f0a1b327fb8aa1880f1639c1e3ed6afbdc |
| A2 | 12570, 9580, 9270, 9220, 11090, 14530, 10620 | false | C:\Users\Dima\AppData\Local\Temp\extractum-process-shell-sessions\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempts\attempt-002\runs\A2.feature-tree.stdout.log | C:\Users\Dima\AppData\Local\Temp\extractum-process-shell-sessions\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempts\attempt-002\timings\A2.diagnostic.dirty.html | 5bf5715827f819407798ee7910ce1818459108f2600bc5b1379fb732b7ffca27 |
| D | 13230, 9810, 9290, 11660, 12470, 10530, 9090 | true | C:\Users\Dima\AppData\Local\Temp\extractum-process-shell-sessions\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempts\attempt-002\runs\D.feature-tree.stdout.log | C:\Users\Dima\AppData\Local\Temp\extractum-process-shell-sessions\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempts\attempt-002\timings\D.diagnostic.dirty.html | 6bd17e290b1b4181a75ea08c5c6270482ab9a05e25f81739bbf4241a8b613cbb |
| A3 | 9220, 9940, 14780, 9360, 9690, 10190, 11010 | false | C:\Users\Dima\AppData\Local\Temp\extractum-process-shell-sessions\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempts\attempt-002\runs\A3.feature-tree.stdout.log | C:\Users\Dima\AppData\Local\Temp\extractum-process-shell-sessions\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempts\attempt-002\timings\A3.diagnostic.dirty.html | 2909fc7fec2353df265a42b77d2ba295535102e957b52deec37673aef03c6d5e |

## Attempt attempt-003 raw measurements

**Recorded kind:** `stability_invalid`

**Recalculated stability reasons:** block_unstable:A0, block_unstable:B, block_unstable:A1, block_unstable:C, block_unstable:D, block_unstable:A3, anchor_range_exceeded

| Block | Wall samples | Median | Within 300 ms | No-op wall | No-op Cargo |
| --- | --- | ---: | ---: | ---: | ---: |
| A0 | 9800.8528, 9454.202, 9262.418, 9647.5704, 14202.2182, 10883.4458, 10645.2607 | 9800.8528 ms | 2/7 | 1306.6154 ms | 1170 ms |
| B | 10713.4033, 11822.6772, 11036.603, 9730.5496, 12566.5081, 11345.1016, 13786.7654 | 11345.1016 ms | 1/7 | 1315.4361 ms | 1200 ms |
| A1 | 9695.3333, 9222.0211, 9496.0867, 9470.2553, 12310.6706, 10110.2797, 12503.7259 | 9695.3333 ms | 3/7 | 1293.9348 ms | 1170 ms |
| C | 13991.4081, 12240.8197, 10722.4369, 12404.933, 9095.6048, 10128.5674, 12400.6176 | 12240.8197 ms | 3/7 | 1459.9585 ms | 1330 ms |
| A2 | 11783.6694, 9265.9326, 9984.5258, 9259.4273, 9111.9948, 9527.998, 9054.1706 | 9265.9326 ms | 5/7 | 1335.9165 ms | 1210 ms |
| D | 13935.5244, 10799.9899, 8972.7251, 15675.6682, 10132.3946, 9645.1635, 12797.4395 | 10799.9899 ms | 1/7 | 1288.9639 ms | 1170 ms |
| A3 | 10787.5939, 10657.3733, 8979.2998, 13279.7313, 10731.5165, 9852.0201, 14660.5656 | 10731.5165 ms | 3/7 | 1313.0556 ms | 1190 ms |

### State evidence

| Block | src-tauri tree | Canonical lib.rs SHA-256 | Metadata direct edge | Cargo target |
| --- | --- | --- | --- | --- |
| A0 | fd9711a041432ef420e7b09d56a46131a2a52a2a | ec21466993001f6d3c76bfc56edb3959478ddca0a3f86a42d278617f4ac89312 | false | G:\Develop\Extractum\.worktrees\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempt-003\src-tauri\target |
| B | 34ba61e94780ac68db1bcb38edc23dfbdcfa44a3 | ec21466993001f6d3c76bfc56edb3959478ddca0a3f86a42d278617f4ac89312 | false | G:\Develop\Extractum\.worktrees\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempt-003\src-tauri\target |
| A1 | fd9711a041432ef420e7b09d56a46131a2a52a2a | ec21466993001f6d3c76bfc56edb3959478ddca0a3f86a42d278617f4ac89312 | false | G:\Develop\Extractum\.worktrees\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempt-003\src-tauri\target |
| C | cb07b39d4f2b2598f76c495f5f0a8287006c2fbf | ec21466993001f6d3c76bfc56edb3959478ddca0a3f86a42d278617f4ac89312 | true | G:\Develop\Extractum\.worktrees\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempt-003\src-tauri\target |
| A2 | fd9711a041432ef420e7b09d56a46131a2a52a2a | ec21466993001f6d3c76bfc56edb3959478ddca0a3f86a42d278617f4ac89312 | false | G:\Develop\Extractum\.worktrees\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempt-003\src-tauri\target |
| D | 77e2d163ccc8bddf3ea051cb995909888cae9aba | c5f678eb13e83050b1a13e0fb34f4fc53310e847889cea551a256066e7c958ce | true | G:\Develop\Extractum\.worktrees\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempt-003\src-tauri\target |
| A3 | fd9711a041432ef420e7b09d56a46131a2a52a2a | ec21466993001f6d3c76bfc56edb3959478ddca0a3f86a42d278617f4ac89312 | false | G:\Develop\Extractum\.worktrees\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempt-003\src-tauri\target |

### Cargo-reported samples and diagnostics

| Block | Cargo durations (ms) | `--extern extractum_process` | Feature graph | Timings HTML | SHA-256 |
| --- | --- | --- | --- | --- | --- |
| A0 | 9680, 9320, 9150, 9530, 14010, 10760, 10530 | false | C:\Users\Dima\AppData\Local\Temp\extractum-process-shell-sessions\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempts\attempt-003\runs\A0.feature-tree.stdout.log | C:\Users\Dima\AppData\Local\Temp\extractum-process-shell-sessions\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempts\attempt-003\timings\A0.diagnostic.dirty.html | a1a4e961e1c330aca695bd60278beaa0a315ae9ec3d25a9f6a7d883dd02bf34f |
| B | 10580, 11630, 10910, 9610, 12440, 11190, 13500 | false | C:\Users\Dima\AppData\Local\Temp\extractum-process-shell-sessions\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempts\attempt-003\runs\B.feature-tree.stdout.log | C:\Users\Dima\AppData\Local\Temp\extractum-process-shell-sessions\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempts\attempt-003\timings\B.diagnostic.dirty.html | 4124ae0c84ae6a6746b79355f165b7946be5f9805a18f2900cbbfc08717949c5 |
| A1 | 9570, 9080, 9370, 9330, 12170, 9980, 12070 | false | C:\Users\Dima\AppData\Local\Temp\extractum-process-shell-sessions\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempts\attempt-003\runs\A1.feature-tree.stdout.log | C:\Users\Dima\AppData\Local\Temp\extractum-process-shell-sessions\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempts\attempt-003\timings\A1.diagnostic.dirty.html | 1c6941af267845839252ba89f4f301263319f492ba22abd35603411c26f9a382 |
| C | 13790, 12100, 10580, 11880, 8970, 10010, 12270 | true | C:\Users\Dima\AppData\Local\Temp\extractum-process-shell-sessions\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempts\attempt-003\runs\C.feature-tree.stdout.log | C:\Users\Dima\AppData\Local\Temp\extractum-process-shell-sessions\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempts\attempt-003\timings\C.diagnostic.dirty.html | ccd63135dcefd351905178c3649967f588c7d55e98a7ee27cbe15e83afc8e086 |
| A2 | 11620, 9150, 9850, 9020, 8990, 9400, 8920 | false | C:\Users\Dima\AppData\Local\Temp\extractum-process-shell-sessions\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempts\attempt-003\runs\A2.feature-tree.stdout.log | C:\Users\Dima\AppData\Local\Temp\extractum-process-shell-sessions\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempts\attempt-003\timings\A2.diagnostic.dirty.html | d5448b67325c4edc7694da1d9d61ccee34934d17f4d2001bd6ebdacf09bdcb2c |
| D | 13810, 10660, 8850, 15500, 9990, 9500, 12660 | true | C:\Users\Dima\AppData\Local\Temp\extractum-process-shell-sessions\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempts\attempt-003\runs\D.feature-tree.stdout.log | C:\Users\Dima\AppData\Local\Temp\extractum-process-shell-sessions\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempts\attempt-003\timings\D.diagnostic.dirty.html | c5a0589b56e712a26eae207a39abc1a0cda7769132dabcb59817a634423b2f15 |
| A3 | 10670, 10530, 8860, 13130, 10610, 9730, 14300 | false | C:\Users\Dima\AppData\Local\Temp\extractum-process-shell-sessions\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempts\attempt-003\runs\A3.feature-tree.stdout.log | C:\Users\Dima\AppData\Local\Temp\extractum-process-shell-sessions\process-shell-session-1d805fa7-86f8-48f4-90ef-b55ef13720b1\attempts\attempt-003\timings\A3.diagnostic.dirty.html | 8dc713d4d2896ccf8f6a3892283477a47298c46ad291aea8b254aface74fb2f5 |

## Decision

No B/C/D/E causal classification is made.

**Unexplained stability-invalid count:** 2

The machine did not support the preregistered 300 ms precision twice. Any next run requires a separately frozen anomaly protocol.

**Required next step:** Keep Phase 4 blocked and preregister a new design with sample count, interleaving/counterbalancing, and stability rule fixed before new data.

## Scope

This diagnostic does not automatically retain `extractum-process` or unblock Phase 4. Any roadmap, threshold, or architecture change remains a separate owner-approved decision.

The result is conditional on the fixed incremental-cache order. Evidence of order-specific hysteresis requires a separately preregistered counterbalanced experiment, not a post-hoc rerun.

