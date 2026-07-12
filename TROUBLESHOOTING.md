# Bevy Open Siege Troubleshooting

Use this runbook when a packaged build fails to start, render, play audio, load saves, or pass release verification. It is written for players, support, and QA so reports contain enough evidence to reproduce a problem without collecting unrelated personal files.

## First Checks

Run these commands from the extracted package directory:

```bash
./verify_release.sh --quick .
./support_diagnostics.sh . support-diagnostics
./bevy_open_siege --print-release-info
./bevy_open_siege --validate-data
./bevy_open_siege --print-save-path
```

If `verify_release.sh --quick .` fails, keep the terminal output and do not edit package files before reporting the issue. The SHA256 check is only meaningful against the original extracted package.

The diagnostics helper writes command output into `support-diagnostics/` without copying save files, screenshots, recordings, crash dumps, or unrelated personal files.

## Window Does Not Open

Run the no-audio startup smoke:

```bash
./runtime_smoke.sh ./bevy_open_siege 12
```

Include `runtime-smoke.txt` if it exists, plus the terminal output. If the output mentions `error[B0001]`, Vulkan, X11, Wayland, missing libraries, or a panic, copy the first panic block and the release info output into the report.

## Blank Or Corrupted Rendering

Run:

```bash
./visual_smoke.sh ./bevy_open_siege 15
```

Attach the generated smoke output and a screenshot if possible. Record the display server, GPU, driver version, monitor resolution, fullscreen state, and whether the issue also happens after pressing `F` to leave fullscreen.

## No Audio Or Audio Device Problems

Audio is opt-in for this release candidate. Start with:

```bash
./audio_smoke.sh ./bevy_open_siege 12
./bevy_open_siege --audio
```

Record whether speakers, headphones, Bluetooth output, or HDMI output were active. Include the command output and whether the window still opens when using the default no-audio launch.

## Save Or Progress Problems

Print the active save path:

```bash
./bevy_open_siege --print-save-path
./bevy_open_siege --print-save-summary
```

For QA, reproduce with an isolated save:

```bash
BEVY_OPEN_SIEGE_SAVE_PATH=qa-save.ron ./bevy_open_siege
```

Do not attach the save file unless the issue requires it and you intentionally choose to share it. `uninstall_linux_user.sh` preserves saves by default; `uninstall_linux_user.sh --purge` removes them.

## Package Or Install Problems

For Linux packages, run:

```bash
./linux_package_audit.sh .
./install_linux_user.sh
./uninstall_linux_user.sh
```

Report the package archive name, checksum from `SHA256SUMS`, command output, shell, Linux distribution, desktop environment, and whether `$XDG_DATA_HOME` or `$XDG_BIN_HOME` is customized.

## Final QA Evidence

Before final approval, initialize the evidence folders and summarize their status:

```bash
./manual_qa_session.sh --init . qa-session
./platform_package_plan.sh --init . platform-session
./qa_evidence_summary.sh --summary . qa-session platform-session
```

Only run `./final_signoff_check.sh --check . qa-session platform-session` after every manual and platform evidence file is marked `Status: Pass` or `Status: Scoped Out` and `Approved: Yes`, and after `QA_SIGNOFF.md` says `Release approved: Yes`.
