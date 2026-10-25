# HANDOFF: g0-upgrade (P8, TPL-3 against embedded_bedrock da9b351)

Done: report at ~/git/tpm/maintain/P2528-embedded-bedrock/design/g0-p8-report.md, sent to parent (g0-template-suite@gpd-omarchy). Plumbing applied: bedrock_fw.json pins da9b351 with the new answers; EVO-12 note in nucleo_g0b1re_api/tests/evolution.rs. Board check of a182dbc OK.

Next: wait for the parent's choice of items 1-9 and the 0_4_0 snapshot; apply (regenerate into a scratch dir: `uv run $TPL/firmware_template_skill/scripts/bedrock_gen.py new --answers bedrock_fw.json --out $TMP/new`), bedrock_fw.json upgrades / rejected / nuances, CHANGELOG [Unreleased], just test --changed, just lint, firmware build, board check with G0B1RE_PROBE=066AFF515754888367185041, git rm HANDOFF.md, tpm work ready.

Gotchas: SiliconRevision Z/Y is a breaking API change (0.5.0), not in G0 despite the P8 input. Probe: never the J-Link 1366:0105 or the donguru c0de:cafe on this PC.
