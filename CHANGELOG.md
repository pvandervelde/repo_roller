# Changelog

## [0.9.0] - 2026-07-06

### Features

- **ci**: consolidate PR checks behind supply chain security gate [4d3ae0af5cfd12c9bd7141599dccf4957fc02b49]
- **ci**: consolidate PR checks behind supply chain security gate (#237) [869aea42851b62c7334123eddd052ba579b7a7d8]


### Bug Fixes

- **api**: update axum path capture syntax from :param to {param} [b16ace4073172300c301bd530747701f5144b275]
- **build,ci**: address clippy error, Dockerfile Rust version, and CI issues [91f279903a7565075d5af35de77e16f607041643]
- **ci**: address PR review comments on release-regent and merge-warden configs [9528fe4a84142a0a3780c53ebd5ee32d9da2fd4c]
- **ci**: address PR review comments on supply-chain security gate workflow [6ace2779ad60c94e1805274229ebfc3d736c233f]
- **ci**: harden release workflow and add supply-chain security [95efca77af1135240b8c7487a9e86b4b90cd55a7]
- **ci**: pin taiki-e/install-action to commit SHA for cargo-audit step [b707f5cbdd59f6800b3288530627528cd38d46c9]
- **ci**: resolve CryptoProvider panics, container startup, and tooling gaps [b5450f466a9004f02500ab3a808fb4d94625e586]
- **ci**: resolve four PR failures [23fc9eaef2b3dd83ea60b840a095d686fcf1dc07]
- **ci**: suppress cargo-audit failures when no patched versions exist [bcffc75c5f4e3f3f0a78c858aa66b09627e80279]
- **deps**: address OSV scan vulnerabilities and consolidate workspace deps [a8592c1a4c567fb284cf02588491761f19d48569]
- **deps**: allow Zlib license in cargo-deny [235568568d4e471b524ecbbe6ce11ac9f63ae980]
- **deps**: enable TLS for git2 HTTPS operations [c46611a0cd1f25ca55a5665f8412169aa194096c]
- **deps**: fix cookie override range and add pnpm-workspace.yaml to prettierignore [cee68dbbd0abce1b3a5e9ab0ee28fcc5fcdfc68c]
- **deps**: update anyhow to 1.0.103 (RUSTSEC-2026-0190) [8798408faee8200b29c4318c48b91b656d7105d3]
- **deps,ci**: resolve cookie CVE and suppress unfixable osv-scanner failures [f801310f0c02e7b8c32514e57412c2e3a183a152]
- **frontend**: reformat +page.server.ts to satisfy prettier 3.9.x [6bf44809360684f1a1bae6b9fba610815822512e]
- **frontend**: resolve eslint-plugin-svelte v3 lint errors [5791fb7a68afdb66b0a63615685d97415c450abc]
- **release**: set version_prefix to 'v' to match existing git tags [ea47e7b58656077d8140210782aa37d06de2abf3]
- **release**: set version_prefix to v to match existing git tags (#244) [7f29e1216833fd7e896c84614dcd302ae47db8ad]
- **template_engine**: remove redundant .into_iter() calls in extend() [81950afd238125b37a2de36d837cde83298fa9ee]


### Tests

- **core,api**: add unit tests, proptests, fuzz targets, and mutation kill tests (#243) [5aa5bc148cdd52358839b47d6c7c5dd7d5820000]
- **core,api,template_engine,config_manager**: add unit test coverage for edge cases [276fe6e62656ff628561aa62d548445807ef62c3]
- **core,template_engine**: add property-based tests with proptest [ced65153a3a68f31159539178e1aa5df4c860b5e]
- **fuzz**: add cargo-fuzz targets for external-input parsers [cb7d407146b4e60956dc1c1a7571cf8c2a09ef84]
- **mutation**: add kill tests for cargo-mutants survivors [7d6c368027f5d88303c97132696e0069a33c041a]
- **review**: address PR review comments [4bfdbea2009a8f537fef7d007854161702e312a8]


### Continuous Integration

- **docs**: pin all action SHAs and pip versions in user-docs job [bac4ef0102fd85db8d12cd0dda4b1e59a7405ca6]
- **docs**: publish user docs to GitHub Pages via MkDocs Material [be4a8469441a16e6a64e26cc1d7f51656d349ac3]
- **docs**: publish user documentation to GitHub Pages via MkDocs Material (#227) [f20797df823e72babc75b151058a59be77e2daaa]
- Deploy user docs to GitHub pages [6cfdcc8c0a2a5f85145f911050d22222489a10f2]
- Fix build errors caused by vitest upgrade [97b981087537031868508efd362d6a8f253d01a5]


### Chores

- **ci**: add release-regent and merge-warden configs, remove superseded workflows [c17dc9b9bf3c2602481d7c25a20e85a18109f12f]
- **ci**: add release-regent and merge-warden configs, remove superseded workflows (#228) [e892c45fc34c706df5d2663e76906e3d9b9fc8ec]
- **deps**: Update the pnpm-lock file [584ad5e2092be8603a4398390f2e5c01b9ab7a9a]
- **deps**: fix Rust OSV vulnerabilities [eafd3fed33ab010e752e51ec1d9ec854747af5dd]
- **deps**: fix npm vulnerabilities in frontend [f2e9bd85fc7bbd867e0d5eb1cd72c09b81e5d103]
- **deps**: lock file maintenance [587da3ea524bb4aa73c713e18a56420162b259f8]
- **deps**: lock file maintenance (#267) [9f7565d49b8cc40bfed825703e253c3fd5965e69]
- **deps**: regenerate Cargo.lock after v0.8.2 version bump [85e5b7a57c74423642e2a2fbebb98f515babf714]
- **deps**: resolve breaking changes from cargo upgrade [de74f945ccae5c3844eb784aa51aa74da43cdeb6]
- **deps**: resolve breaking changes from cargo upgrade (#253) [3b9b89a127b68eac9abfe7da518bcefab5558ec1]
- **deps**: update actions/checkout digest [ffe2db896c9186fd02266d4eef31d10a5f87668d]
- **deps**: update actions/checkout digest (#239) [4a46fe7302dfa09b22e1131e473f7db66c18acf7]
- **deps**: update anthropics/claude-code-action digest to 558b1d6 [7d7491e6566525a510da8ac6463ddaaf2bdc2656]
- **deps**: update anthropics/claude-code-action digest to 558b1d6 (#240) [68c838a4c972a7124a1a97977da54f0c3592679f]
- **deps**: update codecov/codecov-action action to v6 [525b4106ba9b47dd3fbef35874cad6e53a78de5e]
- **deps**: update codecov/codecov-action action to v6 (#248) [2490a7dcaad3d8804a8a9f7bbf2096f5bcd77d2c]
- **deps**: update codecov/codecov-action action to v7 [d70be9f109cfecc0f5bccae602e39b3021489bf4]
- **deps**: update codecov/codecov-action action to v7 (#257) [6b0a18e6ab7ba7df410d5e0849bace9ddcff1e61]
- **deps**: update codecov/codecov-action digest to 0fb7174 [ce1d56fb74cc8827fdaa46194711bff4546f1490]
- **deps**: update codecov/codecov-action digest to 0fb7174 (#234) [554aaaf13067dccb0a9940f339fec34a0c1c418b]
- **deps**: update dependency svelte to v5.55.7 [security] [f492ff4f152f3657ae3a805ce6aa54edab48718a]
- **deps**: update dependency svelte to v5.55.7 [security] (#230) [41e2ea0107620ea393130259dadba10b4e4aafa0]
- **deps**: update dependency vitest to v4 [security] [89bf86fd2b6a9d9a244641426434760dcf0c2bc5]
- **deps**: update dependency vitest to v4 [security] (#246) [b6349ab789737d204aa066a9b4ada4554dbb8764]
- **deps**: update frontend and Rust dependencies [c7fd269ff8fc8d5984bd130998b55067e150e08a]
- **deps**: update frontend and Rust dependencies (#259) [81151366f3c01ea7626b6afc13ca29ee1d9880e9]
- **deps**: update pnpm to v11 [security] [142d16b3e284102555c7a2cb515f2048ffe2cd74]
- **deps**: update pnpm to v11 [security] (#254) [7593e9667ef4045ac5867c7a6c29ba07a44046a1]
- **deps**: update pnpm to v11.8.0 [security] [1edea5fdf5385b26ad909df28700b253f3d7f634]
- **deps**: update pnpm to v11.8.0 [security] (#258) [7dff32d5d93dcc956b2da99d4970c70b79c23d6c]
- **deps**: update rust crate toml to v0.9.12 [2e7c20ad69f74bbac042b9ebe70fab9f55d3dd63]
- **deps**: update rust crate toml to v0.9.12 (#163) [c35a403f8b1b7b8188cae1962526b3431ea0424f]
- **frontend**: apply prettier formatting to pnpm-lock.yaml [45f8ca4d038ef5202b40bec542ea547a14b6418c]
- Add the vscode config to set the window title [2eb9eee803111f4683b4eca153a258d91d979855]
- Add the vscode config to set the window title (#261) [2014e76f94b36fb92fa0e3cbef425dcfb8fdc11f]
- Cargo update [2e03006ff4e092a56d442041d4bae1e1da2ff78b]
- Fix renovate broken config [6ca47b847fcd922a8efab58fe39f530cf7770d2b]
- Fix renovate broken config (#229) [273c577031443c0bc98f102a009c89e5bfed6673]
- Fixes to the configs [a126a9038509aed0db2df5b85621b762750e45c7]
- Fixing build issues [e3b460114f645346be06efea045a9f5556abf5d5]
- Increase Renovate PR number and wait on minimumReleaseAge [0b20fe433b1116ea4b5c9fb7af0d41d88492c8be]
- Increase Renovate PR number and wait on minimumReleaseAge (#250) [8c3ce971f84c294d03aefde864909b2449e744ca]
