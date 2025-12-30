## [0.22.0](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.21.4...v0.22.0) (2025-12-30)


### Features

* **transformer:** implement ResponseTransformer trait for modifying tool responses ([c712001](https://gitlab.com/lx-industries/rmcp-openapi/commit/c712001f94772ccfd49d8a615316ccc5dff65568)), closes [#85](https://gitlab.com/lx-industries/rmcp-openapi/issues/85)


### Bug Fixes

* **ci:** skip branch pipeline for release commits ([7973cfe](https://gitlab.com/lx-industries/rmcp-openapi/commit/7973cfece277cb98830820604d358f6182355dbc))


### Miscellaneous Chores

* **deps:** update rust crate rmcp-actix-web to v0.9.2 ([a85b612](https://gitlab.com/lx-industries/rmcp-openapi/commit/a85b612f76288d231803921691299c58e0c847d8))

## [0.21.4](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.21.3...v0.21.4) (2025-12-29)


### Miscellaneous Chores

* **deps:** update rust crate serde_json to v1.0.148 ([7d8b16e](https://gitlab.com/lx-industries/rmcp-openapi/commit/7d8b16ea52a6a47af8a4874fe0f0cc751f7be3b3))

## [0.21.3](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.21.2...v0.21.3) (2025-12-27)


### Bug Fixes

* prevent stack overflow on self-referencing schemas ([56b5aad](https://gitlab.com/lx-industries/rmcp-openapi/commit/56b5aad6d2050434b30d57ccd650b8613c80b904)), closes [#84](https://gitlab.com/lx-industries/rmcp-openapi/issues/84)


### Miscellaneous Chores

* **deps:** update rust crate jsonschema to 0.38.0 ([5011b63](https://gitlab.com/lx-industries/rmcp-openapi/commit/5011b635c4b4e436feb76a151d75c53cdeaf0153))
* **deps:** update rust crate reqwest to v0.12.28 ([5400346](https://gitlab.com/lx-industries/rmcp-openapi/commit/5400346837dff56fdc79928963e4266296549cc3))
* **deps:** update rust crate schemars to v1.2.0 ([6acc981](https://gitlab.com/lx-industries/rmcp-openapi/commit/6acc981c7b54e03d5706277360e749d0b4dd0d20))
* **deps:** update rust crate serde_json to v1.0.146 ([b05d835](https://gitlab.com/lx-industries/rmcp-openapi/commit/b05d835321e8a020806240e038c9232fb1ef9722))
* **deps:** update rust crate serde_json to v1.0.147 ([3f1ca83](https://gitlab.com/lx-industries/rmcp-openapi/commit/3f1ca833d2e39f1307094ea4431a6e09287854ce))

## [0.21.2](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.21.1...v0.21.2) (2025-12-22)


### Miscellaneous Chores

* **deps:** update rust crate axum to v0.8.8 ([364f568](https://gitlab.com/lx-industries/rmcp-openapi/commit/364f56801f014998508f859b599c8d8c5f5296da))

## [0.21.1](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.21.0...v0.21.1) (2025-12-20)


### Miscellaneous Chores

* **deps:** update rmcp crates ([6d3e5e6](https://gitlab.com/lx-industries/rmcp-openapi/commit/6d3e5e634b1db1e3d5420a7385a8c065d7cd4b8c))
* **deps:** update rust crate insta to v1.45.0 ([3dfec04](https://gitlab.com/lx-industries/rmcp-openapi/commit/3dfec04f98ea376c090c727cee32a2eb765c012c))
* **deps:** update rust crate reqwest to v0.12.26 ([30e0840](https://gitlab.com/lx-industries/rmcp-openapi/commit/30e0840d920feda18afdad59b5f3b301797af9ed))
* **deps:** update rust crate tracing to v0.1.44 ([171fbf8](https://gitlab.com/lx-industries/rmcp-openapi/commit/171fbf8f17729ad07314880a8273eb93a68b79b7))
* **deps:** update rust:1.92.0 docker digest to 48851a8 ([d2ff606](https://gitlab.com/lx-industries/rmcp-openapi/commit/d2ff6060b04ec2bef3d09bfef9ff5c35b0249422))

## [0.21.0](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.20.7...v0.21.0) (2025-12-14)


### Features

* use common http client for all tools to lower memory footprint ([d1e4d58](https://gitlab.com/lx-industries/rmcp-openapi/commit/d1e4d58e40786f63927a3d612141c9677f443c65))


### Miscellaneous Chores

* **deps:** update node.js to 9a2ed90 ([47e85a1](https://gitlab.com/lx-industries/rmcp-openapi/commit/47e85a194e961947333b7f25be40b672d8f5e118))
* **deps:** update node.js to v24.12.0 ([875947b](https://gitlab.com/lx-industries/rmcp-openapi/commit/875947b1d0b9d88a1a90809a0c589cc8efeb2553))
* **deps:** update rust crate reqwest to v0.12.25 ([dc2ad34](https://gitlab.com/lx-industries/rmcp-openapi/commit/dc2ad340520231da2ad6ffdc1b8160f5a6155bff))
* **deps:** update rust docker tag to v1.92.0 ([fe22a3c](https://gitlab.com/lx-industries/rmcp-openapi/commit/fe22a3c4465fee755e10a98ec9d3c6cd3910361e))
* **deps:** update rust:1.91.1 docker digest to 867f1d1 ([b18797a](https://gitlab.com/lx-industries/rmcp-openapi/commit/b18797af9de5dce4c1a194ea7a41deda347c7c31))
* **deps:** upgrade rmcp to 0.11.0 and rmcp-actix-web to 0.9.0 ([8260365](https://gitlab.com/lx-industries/rmcp-openapi/commit/82603651fd5a927bf6f876fda82c3e1fdcadad1b))

## [0.20.7](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.20.6...v0.20.7) (2025-12-08)


### Miscellaneous Chores

* **deps:** update rust crate rmcp to 0.10.0 ([2b926e4](https://gitlab.com/lx-industries/rmcp-openapi/commit/2b926e4fe51c3e011e9cc87bc92bacd6c184b355))
* **deps:** update rust crate rmcp-actix-web to v0.8.18 ([996e144](https://gitlab.com/lx-industries/rmcp-openapi/commit/996e144b807d3fb58c1e82c97a76eb5eed420031))

## [0.20.6](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.20.5...v0.20.6) (2025-12-01)


### Miscellaneous Chores

* **deps:** update rust crate actix-web to v4.12.1 ([8bb1e34](https://gitlab.com/lx-industries/rmcp-openapi/commit/8bb1e3492ed82da364054d1dd648215fc64b9fc7))
* **deps:** update rust crate insta to v1.44.3 ([417f710](https://gitlab.com/lx-industries/rmcp-openapi/commit/417f710174fa3c2923660d8877bf7c49e8b2d97b))
* **deps:** update rust crate jsonschema to v0.37.3 ([ff2faeb](https://gitlab.com/lx-industries/rmcp-openapi/commit/ff2faebabd14d290f5b2b12e3589fb7e6d31cf8b))
* **deps:** update rust crate jsonschema to v0.37.4 ([9ff785c](https://gitlab.com/lx-industries/rmcp-openapi/commit/9ff785cbfb41c2cfe496257ab104d9087c178261))
* **deps:** update rust crate mockito to v1.7.1 ([9d7e683](https://gitlab.com/lx-industries/rmcp-openapi/commit/9d7e683f283c9632ef0e40769ced93749ce640e4))
* **deps:** update tokio-tracing monorepo ([02435be](https://gitlab.com/lx-industries/rmcp-openapi/commit/02435be663eac516b09cbe1e25caa1c0e117c8ad))

## [0.20.5](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.20.4...v0.20.5) (2025-11-26)


### Miscellaneous Chores

* **deps:** update node.js to v24 ([5877acd](https://gitlab.com/lx-industries/rmcp-openapi/commit/5877acd4d8195830f7952e60c2465789c1a6f553))
* **deps:** update rust crate http to v1.4.0 ([31fe65d](https://gitlab.com/lx-industries/rmcp-openapi/commit/31fe65dbfac28b9e175d37d69d0cd4e94de2ee1f))
* **deps:** update rust crate jsonschema to 0.37.0 ([b7c0ed1](https://gitlab.com/lx-industries/rmcp-openapi/commit/b7c0ed19e127106985e51c348d6b2ea57734fc2d))
* **deps:** update rust crate rmcp to 0.9.0 ([ca5c1dd](https://gitlab.com/lx-industries/rmcp-openapi/commit/ca5c1ddfbdb5bb37584c3fc6a088603080cd5547))
* **deps:** update rust crate rmcp-actix-web to v0.8.17 ([044c2c0](https://gitlab.com/lx-industries/rmcp-openapi/commit/044c2c0c319957e2034c90e159cd0cf5adb8cb8d))
* **deps:** update rust:1.91.1 docker digest to 4a29b0d ([a292c23](https://gitlab.com/lx-industries/rmcp-openapi/commit/a292c23e2fc3877f5f2acb25abd3679faa795916))

## [0.20.4](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.20.3...v0.20.4) (2025-11-24)


### Miscellaneous Chores

* **deps:** update node.js to 4ad2c2b ([612af0d](https://gitlab.com/lx-industries/rmcp-openapi/commit/612af0dabfed44e368d22290a382e9403d2d7368))
* **deps:** update rust crate actix-web to v4.12.0 ([2f6774d](https://gitlab.com/lx-industries/rmcp-openapi/commit/2f6774dd0c1a7a446074d40023cba1a86357cec9))
* **deps:** update rust crate axum to v0.8.7 ([455ec4f](https://gitlab.com/lx-industries/rmcp-openapi/commit/455ec4f129a61d050ecf2cc2a129531a59191831))
* **deps:** update rust crate oas3 to 0.20.0 ([43b49f3](https://gitlab.com/lx-industries/rmcp-openapi/commit/43b49f3b8ecb3573fe4125e31e2bf39e50d452e1))
* **deps:** update rust crate rmcp-actix-web to v0.8.14 ([2899b41](https://gitlab.com/lx-industries/rmcp-openapi/commit/2899b41ceb3570c8d417e9a6151ef5d28f493fb9))
* **deps:** update rust docker tag to v1.91.1 ([be4f9a8](https://gitlab.com/lx-industries/rmcp-openapi/commit/be4f9a802e16b7a95fd79a130438561ab4d7d51d))

## [0.20.3](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.20.2...v0.20.3) (2025-11-10)


### Miscellaneous Chores

* **deps:** update node.js to 5cd9156 ([19a4527](https://gitlab.com/lx-industries/rmcp-openapi/commit/19a452708ca98d0b8ae1611a7eaf62e2ca59f760))
* **deps:** update node.js to dcf0610 ([0f33ab7](https://gitlab.com/lx-industries/rmcp-openapi/commit/0f33ab75abfb3c0b34f555b0bea316b9e2af5a4a))
* **deps:** update rust crate rmcp to v0.8.5 ([93322f3](https://gitlab.com/lx-industries/rmcp-openapi/commit/93322f3a03cf899639f05f9a4d898412594d8f70))
* **deps:** update rust crate schemars to v1.1.0 ([3aa5c80](https://gitlab.com/lx-industries/rmcp-openapi/commit/3aa5c80335177423d24c2470f82393c41ba58183))
* **deps:** update rust crate tokio-util to v0.7.17 ([cbb075c](https://gitlab.com/lx-industries/rmcp-openapi/commit/cbb075c6c32124d68045b49a35c3229cf4eadb62))
* **deps:** update rust:1.91.0 docker digest to 087fe68 ([1f31274](https://gitlab.com/lx-industries/rmcp-openapi/commit/1f31274e40b1bf27a4073980c44027b3c3434ca4))
* **deps:** update rust:1.91.0 docker digest to a0dba1c ([86c288a](https://gitlab.com/lx-industries/rmcp-openapi/commit/86c288a3ed09256e213649215d22d5a2208c3cc3))

## [0.20.2](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.20.1...v0.20.2) (2025-11-03)


### Miscellaneous Chores

* **deps:** update node.js to v22.21.1 ([ce71388](https://gitlab.com/lx-industries/rmcp-openapi/commit/ce71388d0468f46b3614501ed830ea51656b2cff))
* **deps:** update rust crate clap to v4.5.51 ([01c23f0](https://gitlab.com/lx-industries/rmcp-openapi/commit/01c23f0034be45acaeee02c5e978b0fc69e23e0d))
* **deps:** update rust crate schemars to v1.0.5 ([56d9924](https://gitlab.com/lx-industries/rmcp-openapi/commit/56d992475ea98c1c2d5ce0e900dbc29f42301e2e))
* **deps:** update rust docker tag to v1.91.0 ([e118bca](https://gitlab.com/lx-industries/rmcp-openapi/commit/e118bcaa81b3625cf58c3aff0eac24a4a02295c8))
* **deps:** update rust:1.90.0 docker digest to e227f20 ([a69657e](https://gitlab.com/lx-industries/rmcp-openapi/commit/a69657e9e6b25f8414f6a72ce2ad4d3f5d145c56))

## [0.20.1](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.20.0...v0.20.1) (2025-10-28)


### Miscellaneous Chores

* **deps:** update rust crate rmcp-actix-web to v0.8.11 ([e7a065b](https://gitlab.com/lx-industries/rmcp-openapi/commit/e7a065bd28d9e29ae63714f10ee5bde037853fef))

## [0.20.0](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.19.4...v0.20.0) (2025-10-27)


### Features

* builder pattern for the new filter struct ([ca683e8](https://gitlab.com/lx-industries/rmcp-openapi/commit/ca683e8fa91456f75daf43c9c4399d6db6d91402))
* filtering by OperationId, refactoring of the filtering models ([9bb65a7](https://gitlab.com/lx-industries/rmcp-openapi/commit/9bb65a75900486022a3b858cdd77263562f91e2e))

## [0.19.4](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.19.3...v0.19.4) (2025-10-27)


### Miscellaneous Chores

* **deps:** update node.js to 915acd9 ([badc757](https://gitlab.com/lx-industries/rmcp-openapi/commit/badc757a2dc0a5de9a1cb05cbc802ff7ad7404a6))
* **deps:** update node.js to v22.21.0 ([d35cf37](https://gitlab.com/lx-industries/rmcp-openapi/commit/d35cf37f71fad920b87bd37447a8d182bb770d43))
* **deps:** update rust crate clap to v4.5.50 ([a704426](https://gitlab.com/lx-industries/rmcp-openapi/commit/a704426d1a346e495b10b4bac65d1263a1d0c4a5))
* **deps:** update rust crate rmcp to v0.8.3 ([16095c1](https://gitlab.com/lx-industries/rmcp-openapi/commit/16095c17873c53788990a7dc8901e0f934bbf78f))
* **deps:** update rust crate rmcp-actix-web to v0.8.10 ([7bbbd0b](https://gitlab.com/lx-industries/rmcp-openapi/commit/7bbbd0b4958d470f2c4c862580f7521df2bc202a))
* **deps:** update rust:1.90.0 docker digest to 52e36cd ([3ff28ef](https://gitlab.com/lx-industries/rmcp-openapi/commit/3ff28ef84ea0f5ca9e61c2efe2eba7f8a3a638da))

## [0.19.3](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.19.2...v0.19.3) (2025-10-20)


### Miscellaneous Chores

* **deps:** update rust crate indexmap to v2.12.0 ([e74fd98](https://gitlab.com/lx-industries/rmcp-openapi/commit/e74fd98f4cb9d7138b21b42cfa66d8deb40213fc))
* **deps:** update rust crate tokio to v1.48.0 ([98a3f61](https://gitlab.com/lx-industries/rmcp-openapi/commit/98a3f61257711cd6dce7826ec499ebd65c50c2e7))

## [0.19.2](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.19.1...v0.19.2) (2025-10-15)


### Miscellaneous Chores

* **deps:** update rust crate clap to v4.5.49 ([563d587](https://gitlab.com/lx-industries/rmcp-openapi/commit/563d58772eac4f90f69d9cc1e0d8907aef57dead))
* **deps:** update rust crate regex to v1.12.2 ([ad3a138](https://gitlab.com/lx-industries/rmcp-openapi/commit/ad3a13855302163a0d8788b6054ec224d5c229e3))
* **deps:** update rust crate reqwest to v0.12.24 ([40e5197](https://gitlab.com/lx-industries/rmcp-openapi/commit/40e5197abb29083665f509559bf01124fed987f7))
* **deps:** update rust crate rmcp-actix-web to v0.8.9 ([a693588](https://gitlab.com/lx-industries/rmcp-openapi/commit/a693588c9409d75b60c20b2ac046af68e2ef4b18))

## [0.19.1](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.19.0...v0.19.1) (2025-10-13)


### Bug Fixes

* **http_client:** merge base_url with path correctly ([c623a03](https://gitlab.com/lx-industries/rmcp-openapi/commit/c623a0321ea386e007939c5b4ea36948d3fad76a))


### Miscellaneous Chores

* **deps:** update rust crate regex to v1.12.1 ([c63a649](https://gitlab.com/lx-industries/rmcp-openapi/commit/c63a6492583894b7804fae9927b46fbec81c5ba6))

## [0.19.0](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.18.0...v0.19.0) (2025-10-10)


### Features

* add MCP tool annotations based on HTTP method semantics ([2c8e47e](https://gitlab.com/lx-industries/rmcp-openapi/commit/2c8e47e4efb5b57f22e3142aa655f56439d2413d)), closes [#73](https://gitlab.com/lx-industries/rmcp-openapi/issues/73)
* add support for MCP tool image responses ([cdac021](https://gitlab.com/lx-industries/rmcp-openapi/commit/cdac021b3c2dfacaf84219c5efaff43905bd5e86))


### Bug Fixes

* return error when image Content-Type header is missing ([e6ba324](https://gitlab.com/lx-industries/rmcp-openapi/commit/e6ba324cc9af41657aa63d679883f6804e71588a))
* suppress [secure] positive dead code warnings in test mocks ([0fbbcd7](https://gitlab.com/lx-industries/rmcp-openapi/commit/0fbbcd7d1e9f9258c7c1f5f77797c593e541da5a))
* **tests:** fix outdated SSE integration test snapshots ([29d56ae](https://gitlab.com/lx-industries/rmcp-openapi/commit/29d56aed58bd379b400bd4658756639d4959438f))


### Miscellaneous Chores

* **deps:** update rust crate bon to v3.8.1 ([0d0e1f8](https://gitlab.com/lx-industries/rmcp-openapi/commit/0d0e1f88f371d03bf7954995662eb0d6ebf7ddab))
* **deps:** update rust crate rmcp to v0.8.1 ([5e5437d](https://gitlab.com/lx-industries/rmcp-openapi/commit/5e5437d1fccfe447116232d144acbdb01567e11f))

## [0.18.0](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.17.0...v0.18.0) (2025-10-06)


### Features

* more consistent messages for null value validation errors ([3b0c5fc](https://gitlab.com/lx-industries/rmcp-openapi/commit/3b0c5fc49007cb488af55493036ce6f9543d77a1))

## [0.17.0](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.16.5...v0.17.0) (2025-10-06)


### Features

* add context-aware null parameter error messages ([04a5074](https://gitlab.com/lx-industries/rmcp-openapi/commit/04a5074fa2c1ec7e9f2a0414d4441a98cf263d7c)), closes [#78](https://gitlab.com/lx-industries/rmcp-openapi/issues/78)
* add server-side parameter mapping storage ([28d292c](https://gitlab.com/lx-industries/rmcp-openapi/commit/28d292c54d138704843cf7d7debb24b8d9356a99))
* remove schema annotations from generated schemas ([fb1388f](https://gitlab.com/lx-industries/rmcp-openapi/commit/fb1388f35d4322c9dbb94a1d71335a1aa5b3d5bf))

## [0.16.5](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.16.4...v0.16.5) (2025-10-06)


### Miscellaneous Chores

* **deps:** update rust crate bon to v3.8.0 ([70fa510](https://gitlab.com/lx-industries/rmcp-openapi/commit/70fa510bd30fa6967069e5fdf8cd34838fc323c7))
* **deps:** update rust crate rmcp-actix-web to v0.8.8 ([4d2a5d8](https://gitlab.com/lx-industries/rmcp-openapi/commit/4d2a5d8b37a6506a9f32215e1bf20878767c047d))
* **deps:** update rust:1.90.0 docker digest to 976303c ([232f191](https://gitlab.com/lx-industries/rmcp-openapi/commit/232f191d2c9e158a81f987da4ae8dc0d730882b3))

## [0.16.4](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.16.3...v0.16.4) (2025-10-04)


### Miscellaneous Chores

* **deps:** update rust crate rmcp to 0.8.0 ([1514990](https://gitlab.com/lx-industries/rmcp-openapi/commit/151499080fffe09e83f0e8107674f0eea94b4fdb))

## [0.16.3](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.16.2...v0.16.3) (2025-10-04)


### Miscellaneous Chores

* **deps:** update node.js to 2bb201f ([fd1c194](https://gitlab.com/lx-industries/rmcp-openapi/commit/fd1c1943b0f2bb06b951e14d3c54a4a977db672b))
* **deps:** update rust crate axum to v0.8.6 ([015a349](https://gitlab.com/lx-industries/rmcp-openapi/commit/015a34948615b2b050bfda91bd10d82dcffd8970))
* **deps:** update rust crate rmcp-actix-web to v0.8.5 ([f86963e](https://gitlab.com/lx-industries/rmcp-openapi/commit/f86963e4c202e5f2306a790e2fd3fe49e8e3c5a0))
* **deps:** update rust crate rmcp-actix-web to v0.8.6 ([aa03cf7](https://gitlab.com/lx-industries/rmcp-openapi/commit/aa03cf7972faa43cf2631c0a6824acc370a50cf9))
* **deps:** update rust crate thiserror to v2.0.17 ([16053be](https://gitlab.com/lx-industries/rmcp-openapi/commit/16053be5a9ed1b64f3d9c51e989a6ce3b3de2765))
* **deps:** update rust:1.90.0 docker digest to 512d81e ([18e61fe](https://gitlab.com/lx-industries/rmcp-openapi/commit/18e61fe5fca65a048a12d4cad999c66d09f3b42f))

## [0.16.2](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.16.1...v0.16.2) (2025-09-29)


### Miscellaneous Chores

* **deps:** update node.js to v22.20.0 ([c433307](https://gitlab.com/lx-industries/rmcp-openapi/commit/c433307507716fbe4133c75b6f677aa443455dd4))
* **deps:** update rust crate axum to v0.8.5 ([bc85801](https://gitlab.com/lx-industries/rmcp-openapi/commit/bc85801cb3335db36c5a9772f51d2cd409febd68))
* **deps:** update rust crate regex to v1.11.3 ([0738473](https://gitlab.com/lx-industries/rmcp-openapi/commit/0738473f2dd320a38b06d46def4c934856c3eb87))
* **deps:** update rust crate serde to v1.0.228 ([043d64b](https://gitlab.com/lx-industries/rmcp-openapi/commit/043d64b066976ac64f47dd7a1ee3acfc49d34f2d))

## [0.16.1](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.16.0...v0.16.1) (2025-09-25)


### Bug Fixes

* update rmcp to 0.7.0 with required features and dependencies ([c09be17](https://gitlab.com/lx-industries/rmcp-openapi/commit/c09be175c5aa827041842aa46c061e2ab6e5bd97)), closes [#76](https://gitlab.com/lx-industries/rmcp-openapi/issues/76)

## [0.16.0](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.15.2...v0.16.0) (2025-09-24)


### Features

* add support to skip parameter descriptions to save on tokens ([961e58d](https://gitlab.com/lx-industries/rmcp-openapi/commit/961e58d639ee97531d43e290b81510a8a67af902))
* add support to skip tool descriptions to save on tokens ([8d2f0c7](https://gitlab.com/lx-industries/rmcp-openapi/commit/8d2f0c75d14d886d44302e8097148a06e47958d9))
* enable support for system root TLS certificates ([32c7d3d](https://gitlab.com/lx-industries/rmcp-openapi/commit/32c7d3dd2ded7a6452cec7bbd4293e71ae358a75))


### Miscellaneous Chores

* **deps:** update rust crate rmcp to v0.6.4 ([9b9448d](https://gitlab.com/lx-industries/rmcp-openapi/commit/9b9448d1f4578e6ee31e149d6131b6de458fea38))
* **deps:** update rust crate rmcp-actix-web to v0.8.3 ([471c343](https://gitlab.com/lx-industries/rmcp-openapi/commit/471c343828220408186ef7a97e1bb5ffebe2e235))

## [0.15.2](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.15.1...v0.15.2) (2025-09-22)


### Miscellaneous Chores

* **deps:** update rust crate anyhow to v1.0.100 ([b842ca8](https://gitlab.com/lx-industries/rmcp-openapi/commit/b842ca818e60b89e52323ae4d4a314bdaf0cda17))
* **deps:** update rust crate clap to v4.5.48 ([96176b9](https://gitlab.com/lx-industries/rmcp-openapi/commit/96176b9401390253d374452ebf6d990dbdd80c53))
* **deps:** update rust crate indexmap to v2.11.3 ([b46585b](https://gitlab.com/lx-industries/rmcp-openapi/commit/b46585ba9c03357c605c80384f5575ad4f5c56b7))
* **deps:** update rust crate indexmap to v2.11.4 ([cce9d6a](https://gitlab.com/lx-industries/rmcp-openapi/commit/cce9d6a3e74be2b00adde23819e45384e7d2f3a4))
* **deps:** update rust crate rmcp-actix-web to v0.8.2 ([72c9e0e](https://gitlab.com/lx-industries/rmcp-openapi/commit/72c9e0ee6f744b588f2e57c3a0a9aacfd14afa8d))
* **deps:** update rust crate serde to v1.0.225 ([2373ccb](https://gitlab.com/lx-industries/rmcp-openapi/commit/2373ccbb271f0587bc1d161239e9ee8f18779821))
* **deps:** update rust crate serde to v1.0.226 ([2ee23c5](https://gitlab.com/lx-industries/rmcp-openapi/commit/2ee23c599efbff5c02ed0d4366749d7966a5ae0e))
* **deps:** update rust docker tag to v1.90.0 ([e2f59e9](https://gitlab.com/lx-industries/rmcp-openapi/commit/e2f59e958305a82006ea96c30742afc9adeeb2dc))
* **deps:** update rust:1.89.0 docker digest to 57407b3 ([f2429f2](https://gitlab.com/lx-industries/rmcp-openapi/commit/f2429f2316d9c97b6b27b46e3752072a8e239d85))

## [0.15.1](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.15.0...v0.15.1) (2025-09-15)


### Miscellaneous Chores

* **deps:** update rust crate serde to v1.0.220 ([d1c5cf2](https://gitlab.com/lx-industries/rmcp-openapi/commit/d1c5cf20619eced97c8cd4c7ff7a6ac3a3952882))
* **deps:** update rust crate serde to v1.0.223 ([87cdb95](https://gitlab.com/lx-industries/rmcp-openapi/commit/87cdb957d2d9625d8cfcc68b466e0f3749a8e459))
* **deps:** update rust crate serde_json to v1.0.145 ([200713d](https://gitlab.com/lx-industries/rmcp-openapi/commit/200713d1bba01b74aa7d8e93a0e4d4edc0ba1cbf))

## [0.15.0](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.14.0...v0.15.0) (2025-09-12)


### Features

* add server_title field to Server struct ([ae8864f](https://gitlab.com/lx-industries/rmcp-openapi/commit/ae8864f3bf2cd98bd9509e2799a48a6f83412a7c)), closes [#26](https://gitlab.com/lx-industries/rmcp-openapi/issues/26) [#26](https://gitlab.com/lx-industries/rmcp-openapi/issues/26)


### Miscellaneous Chores

* **deps:** update rust crate indexmap to v2.11.1 ([018dca1](https://gitlab.com/lx-industries/rmcp-openapi/commit/018dca1867731d22948ebae853d4afa08b6e037e))
* **deps:** update rust crate rmcp-actix-web to v0.8.1 ([08ca765](https://gitlab.com/lx-industries/rmcp-openapi/commit/08ca765193da7f53bfd106fc64349abb12fce805))

## [0.14.0](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.13.0...v0.14.0) (2025-09-11)


### Features

* gate SSE transport behind transport-sse cargo feature ([84b6cf3](https://gitlab.com/lx-industries/rmcp-openapi/commit/84b6cf3ddfe35fcd579bef5a9425413d6fb1e92a))
* make server info customizable using OpenAPI metadata ([4237cef](https://gitlab.com/lx-industries/rmcp-openapi/commit/4237cef4720b3d40275de331917c1919ce1a05d5))


### Miscellaneous Chores

* **deps:** update node.js to afff6d8 ([5bc3f99](https://gitlab.com/lx-industries/rmcp-openapi/commit/5bc3f99fc5fe550b385521e96ef7db180a47bafc))
* **deps:** update node.js to d6ba961 ([e66af6a](https://gitlab.com/lx-industries/rmcp-openapi/commit/e66af6aa318ae86f064960a08a18fb93e048c02a))
* **deps:** update rust:1.89.0 docker digest to 1ca9500 ([b54d676](https://gitlab.com/lx-industries/rmcp-openapi/commit/b54d676afc25ed45b7597eeceef8780141b90794))
* **deps:** update rust:1.89.0 docker digest to 9e1b362 ([a328be9](https://gitlab.com/lx-industries/rmcp-openapi/commit/a328be992fb4d287bcbf62485c5237ea6d55cf53))

## [0.13.0](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.12.1...v0.13.0) (2025-09-08)


### Features

* add the rustls-tls (enabled by default) and native-tls Cargo features ([b459983](https://gitlab.com/lx-industries/rmcp-openapi/commit/b4599832ce9d28e8453b67b2a0775a90c601631f))

## [0.12.1](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.12.0...v0.12.1) (2025-09-05)


### Miscellaneous Chores

* **deps:** update rust crate insta to v1.43.2 ([7ff16f1](https://gitlab.com/lx-industries/rmcp-openapi/commit/7ff16f100e8cf4b71df473ef93e545825375d5d3))
* **deps:** update rust crate rmcp to v0.6.3 ([b39a684](https://gitlab.com/lx-industries/rmcp-openapi/commit/b39a684a08fda5f31c0060b7556e6005b6747247))
* **deps:** update rust crate rmcp-actix-web to v0.6.1 ([eb08f4a](https://gitlab.com/lx-industries/rmcp-openapi/commit/eb08f4a0034c262edc2093665eecd19c0e86da5e))

## [0.12.0](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.11.0...v0.12.0) (2025-09-04)


### Features

* add configurable authorization modes for token passthrough ([322c99c](https://gitlab.com/lx-industries/rmcp-openapi/commit/322c99c3d3afedcd3fa79b43f0cf91c052779406)), closes [#67](https://gitlab.com/lx-industries/rmcp-openapi/issues/67)

## [0.11.0](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.10.2...v0.11.0) (2025-09-04)


### Features

* add Authorization header pass-through for MCP to OpenAPI bridge ([2aef664](https://gitlab.com/lx-industries/rmcp-openapi/commit/2aef6641dfa3eb94d57a2f9f37ea045508aa54ae))


### Bug Fixes

* add preserve_order feature to serde_json for deterministic JSON key ordering ([b70b1d9](https://gitlab.com/lx-industries/rmcp-openapi/commit/b70b1d9e2b853e13208f2d19d2fc884be9ed0419))


### Miscellaneous Chores

* **deps:** update rust crate bon to v3.7.2 ([eccce01](https://gitlab.com/lx-industries/rmcp-openapi/commit/eccce01b30833834d0ebcd69398696955445bf31))
* **deps:** update rust crate clap to v4.5.47 ([81668f2](https://gitlab.com/lx-industries/rmcp-openapi/commit/81668f241f93853f1d8cd3ad289b052fca8676f2))

## [0.10.2](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.10.1...v0.10.2) (2025-09-01)


### Miscellaneous Chores

* **deps:** update node.js to v22.19.0 ([548512c](https://gitlab.com/lx-industries/rmcp-openapi/commit/548512c390f70f5a77434b851a725bc8ace9bd95))
* **deps:** update rust crate tracing-subscriber to v0.3.20 ([b81066b](https://gitlab.com/lx-industries/rmcp-openapi/commit/b81066b22c23cf2dbeb2ae343ccd1ffddbe7e242))

## [0.10.1](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.10.0...v0.10.1) (2025-08-30)


### Miscellaneous Chores

* **deps:** update rust crate rmcp to v0.6.1 ([ddaba7d](https://gitlab.com/lx-industries/rmcp-openapi/commit/ddaba7d66be4c717acbe09b63ef3528b4f092986))
* **deps:** update rust:1.89.0 docker digest to 3329e2d ([2bfb87c](https://gitlab.com/lx-industries/rmcp-openapi/commit/2bfb87c62fb96a9f1f1d40e1ea78e3811b1c118f))

## [0.10.0](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.9.0...v0.10.0) (2025-08-29)


### Features

* implement proper optional array parameter handling in HTTP requests ([b90c893](https://gitlab.com/lx-industries/rmcp-openapi/commit/b90c89354649e8c3140bfbb487b47d33415b78bf))
* make base_url mandatory in Server struct ([1eca579](https://gitlab.com/lx-industries/rmcp-openapi/commit/1eca579e5bfd6dbed351eacfc4fd9eefbacb3ec1))
* refactor Server/Configuration to eliminate field duplication ([6af0b88](https://gitlab.com/lx-industries/rmcp-openapi/commit/6af0b884124a743c73c1d61bf71d3d9506a202a2))
* restore builder pattern for Server struct ([90b8bc7](https://gitlab.com/lx-industries/rmcp-openapi/commit/90b8bc77f8de0def4c1252498c95d1830c322bb4))


### Miscellaneous Chores

* **deps:** update rust:1.89.0 docker digest to 26318ae ([c506ac4](https://gitlab.com/lx-industries/rmcp-openapi/commit/c506ac4ba545c8a30e2494541d9d0313c1826278))

## [0.9.0](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.8.3...v0.9.0) (2025-08-27)


### Features

* upgrade oas3 to 0.19.0 and leverage reference metadata fields ([b5b2552](https://gitlab.com/lx-industries/rmcp-openapi/commit/b5b25529e3aa3a634c6c63bfaf315b085df82b7a))


### Miscellaneous Chores

* **deps:** update rust crate clap to v4.5.46 ([4d58ffe](https://gitlab.com/lx-industries/rmcp-openapi/commit/4d58ffefddfa9276fa928f455f5ed5a3e91db419))

## [0.8.3](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.8.2...v0.8.3) (2025-08-26)


### Bug Fixes

* broken README code examples ([2146c3c](https://gitlab.com/lx-industries/rmcp-openapi/commit/2146c3c40b1422583baf6e7d72eef2fe958c2128)), closes [#55](https://gitlab.com/lx-industries/rmcp-openapi/issues/55)

## [0.8.2](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.8.1...v0.8.2) (2025-08-25)


### Miscellaneous Chores

* **deps:** update rust crate indexmap to v2.11.0 ([a399498](https://gitlab.com/lx-industries/rmcp-openapi/commit/a39949841131a110099cad9f735a726fefb07595))
* **deps:** update rust crate jsonschema to 0.33.0 ([2ba2be0](https://gitlab.com/lx-industries/rmcp-openapi/commit/2ba2be02c7b433a4cedd573770fc35c35b87582c))
* **deps:** update rust crate regex to v1.11.2 ([ae4c26a](https://gitlab.com/lx-industries/rmcp-openapi/commit/ae4c26a7d6f3665766f1a3e57ae96dc31fb3325c))
* **deps:** update rust crate url to v2.5.6 ([6420f03](https://gitlab.com/lx-industries/rmcp-openapi/commit/6420f03cb13ff2b3eb923af4ac19e22356f60a53))
* **deps:** update rust crate url to v2.5.7 ([0360f71](https://gitlab.com/lx-industries/rmcp-openapi/commit/0360f715636d6a00251a59f6ed8682c3a822b14c))

## [0.8.1](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.8.0...v0.8.1) (2025-08-21)


### Miscellaneous Chores

* **deps:** update rust crate bon to v3.7.1 ([c853168](https://gitlab.com/lx-industries/rmcp-openapi/commit/c853168d1e44f8d384d0cefc49bdee40e866b439))
* **deps:** update rust crate rmcp to v0.6.0 ([36fb211](https://gitlab.com/lx-industries/rmcp-openapi/commit/36fb211591ad3ca8c225be01f41765dc0b78a4f7))
* **deps:** update rust crate thiserror to v2.0.16 ([0f36881](https://gitlab.com/lx-industries/rmcp-openapi/commit/0f368813d8dbff09e82d9a743c538863d5276a26))
* **deps:** update rust:1.89.0 docker digest to 6e6d04b ([309e8bd](https://gitlab.com/lx-industries/rmcp-openapi/commit/309e8bde2844285defe4379f429c0a7da3b871b4))

## [0.8.0](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.7.0...v0.8.0) (2025-08-19)


### Features

* implement structured logging with tracing crate ([98f9faf](https://gitlab.com/lx-industries/rmcp-openapi/commit/98f9faf401af76777c3295af32d3fe8ffe9b617d)), closes [#16](https://gitlab.com/lx-industries/rmcp-openapi/issues/16)


### Bug Fixes

* add delay after npm install to prevent Node.js export maps race condition ([67d4f46](https://gitlab.com/lx-industries/rmcp-openapi/commit/67d4f46b518b4258e936ce008df1ff5e38eb2bfd)), closes [#54](https://gitlab.com/lx-industries/rmcp-openapi/issues/54)
* pin exact SDK version for better CI reproducibility ([79c0fbb](https://gitlab.com/lx-industries/rmcp-openapi/commit/79c0fbb6d3ce0a5b5a80ca370eeb4cee54397749)), closes [#54](https://gitlab.com/lx-industries/rmcp-openapi/issues/54)


### Miscellaneous Chores

* **deps:** update rust crate serde_json to v1.0.143 ([50875a1](https://gitlab.com/lx-industries/rmcp-openapi/commit/50875a15a618d91f046f272b2dee663bd720fb3a))
* **deps:** update rust crate thiserror to v2.0.15 ([0a90724](https://gitlab.com/lx-industries/rmcp-openapi/commit/0a90724c389b568d7c025a6f99fb22a3cab02cfb))

## [0.7.0](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.6.1...v0.7.0) (2025-08-18)


### Features

* expose OpenAPI endpoints as individual MCP tools ([47a1b34](https://gitlab.com/lx-industries/rmcp-openapi/commit/47a1b34850cb03fa248575f43a90d86d9ff11f80))


### Miscellaneous Chores

* **deps:** update node.js to 3266bc9 ([dacf99a](https://gitlab.com/lx-industries/rmcp-openapi/commit/dacf99adfe212aabb37ae7f86712304521e62440))
* **deps:** update node.js to 5cc5271 ([6228e61](https://gitlab.com/lx-industries/rmcp-openapi/commit/6228e61d40ec66c7fbc6402269a13f4f35464022))
* **deps:** update rust crate anyhow to v1.0.99 ([f5b9ee6](https://gitlab.com/lx-industries/rmcp-openapi/commit/f5b9ee6ffc0c4e8d36eb1e5ea4b4e378c316329a))
* **deps:** update rust crate clap to v4.5.45 ([afc6801](https://gitlab.com/lx-industries/rmcp-openapi/commit/afc6801d9f7dac995c523f3d8b0d52626065b0e5))
* **deps:** update rust crate reqwest to v0.12.23 ([46996cc](https://gitlab.com/lx-industries/rmcp-openapi/commit/46996cc9812396584f8046f2938a10ec2e1fddd4))
* **deps:** update rust:1.89.0 docker digest to 5fa1490 ([9c784cc](https://gitlab.com/lx-industries/rmcp-openapi/commit/9c784cc3f426fa08eeb8a3e8328ed6c6e9dc1968))
* **deps:** update rust:1.89.0 docker digest to ded0544 ([cc8d08f](https://gitlab.com/lx-industries/rmcp-openapi/commit/cc8d08f6b071b91e644fefb6b3ebe541805c8efb))
* **deps:** update rust:1.89.0 docker digest to e090f7b ([792cc9c](https://gitlab.com/lx-industries/rmcp-openapi/commit/792cc9cfc12d1ca2e6ef4dfe90e44eb4c773d8b4))

## [0.6.1](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.6.0...v0.6.1) (2025-08-11)


### Miscellaneous Chores

* **deps:** update rust crate rmcp to 0.5.0 ([e87f7a4](https://gitlab.com/lx-industries/rmcp-openapi/commit/e87f7a43e65fb88343b19972d7ddfda092b65c9e))
* **deps:** update rust docker tag to v1.89.0 ([22fd436](https://gitlab.com/lx-industries/rmcp-openapi/commit/22fd4363ea3bbaf05e58fb16953ddf33cdacf0bb))

## [0.6.0](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.5.1...v0.6.0) (2025-08-07)


### Features

* add --header CLI option for authentication support ([0f83763](https://gitlab.com/lx-industries/rmcp-openapi/commit/0f837634f99c72a9e0f97f5d8259632044948ef9)), closes [#43](https://gitlab.com/lx-industries/rmcp-openapi/issues/43)
* add --methods CLI option for filtering OpenAPI operations by HTTP methods ([03669db](https://gitlab.com/lx-industries/rmcp-openapi/commit/03669db938d248d224c821f802dce6443d6a8e71)), closes [#50](https://gitlab.com/lx-industries/rmcp-openapi/issues/50)
* add --tags CLI flag for filtering OpenAPI operations by tags ([f441235](https://gitlab.com/lx-industries/rmcp-openapi/commit/f4412358d2959102f8deb0a3f1140832db703b53)), closes [#48](https://gitlab.com/lx-industries/rmcp-openapi/issues/48)
* add user-agent header with dynamic version ([1de7c7b](https://gitlab.com/lx-industries/rmcp-openapi/commit/1de7c7bd25405bf09c11ae6cac730b5f184f3c5a)), closes [#47](https://gitlab.com/lx-industries/rmcp-openapi/issues/47)
* implement kebab-case normalization for --tags CLI option ([56bf100](https://gitlab.com/lx-industries/rmcp-openapi/commit/56bf1003100d7b4166276c54d01c143fd52e416c)), closes [#49](https://gitlab.com/lx-industries/rmcp-openapi/issues/49)
* implement OpenAPI explode property support for array query parameters ([319b8c0](https://gitlab.com/lx-industries/rmcp-openapi/commit/319b8c0d7c4905dfc9e526b6466e2b8130c47c7b))


### Miscellaneous Chores

* **deps:** update rust crate clap to v4.5.43 ([17cbad2](https://gitlab.com/lx-industries/rmcp-openapi/commit/17cbad23f0ac394cc316151bf41383abcf631e5d))
* **deps:** update rust crate rmcp to v0.4.1 ([1907130](https://gitlab.com/lx-industries/rmcp-openapi/commit/19071304b642695532d911cb32e09314e53bcfdd))

## [0.5.1](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.5.0...v0.5.1) (2025-08-06)


### Miscellaneous Chores

* **deps:** update node.js to v22.18.0 ([099a32a](https://gitlab.com/lx-industries/rmcp-openapi/commit/099a32aa9e858fec76d8afaf523a2c12f95b20eb))
* **deps:** update rust crate rmcp to 0.4.0 ([cf641d7](https://gitlab.com/lx-industries/rmcp-openapi/commit/cf641d7cd1f88f068461dfd3b10cf99d9e96815c))
* **deps:** update rust crate tokio-util to v0.7.16 ([d0029c4](https://gitlab.com/lx-industries/rmcp-openapi/commit/d0029c4fe8f85cd80cc89f0f59afc8c2b2cd410f))

## [0.5.0](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.4.0...v0.5.0) (2025-08-04)


### Features

* add OpenAPI parameter example fields to MCP tool schemas ([5818243](https://gitlab.com/lx-industries/rmcp-openapi/commit/5818243f4c55c5d051bc8bfaa75a9a20c9500726)), closes [#39](https://gitlab.com/lx-industries/rmcp-openapi/issues/39)
* improve format_examples_for_description() to preserve example fidelity ([8eee272](https://gitlab.com/lx-industries/rmcp-openapi/commit/8eee272b821d34e53059aa525bd39e2e21772e0a))
* improve validation error messages with Display trait and better formatting ([2ef5213](https://gitlab.com/lx-industries/rmcp-openapi/commit/2ef521388d41b386c735848e7156463f594cdb13)), closes [#38](https://gitlab.com/lx-industries/rmcp-openapi/issues/38)
* include parameter examples in descriptions for better MCP tool usability ([18d18a0](https://gitlab.com/lx-industries/rmcp-openapi/commit/18d18a003643089eea028722a2305a668aaf39dc)), closes [#40](https://gitlab.com/lx-industries/rmcp-openapi/issues/40)


### Miscellaneous Chores

* **deps:** update rust crate clap to v4.5.42 ([024d514](https://gitlab.com/lx-industries/rmcp-openapi/commit/024d51407ef4c1babf68c8f7d33d5cacb8c5b66f))
* **deps:** update rust crate jsonschema to 0.31.0 ([6846b8d](https://gitlab.com/lx-industries/rmcp-openapi/commit/6846b8dac507586a1aeafb942cc96d063f86f112))
* **deps:** update rust crate jsonschema to 0.32.0 ([8cb1ed1](https://gitlab.com/lx-industries/rmcp-openapi/commit/8cb1ed156804b51a9d1235c5137270837fc10dac))
* **deps:** update rust crate jsonschema to v0.32.1 ([2d5b9a1](https://gitlab.com/lx-industries/rmcp-openapi/commit/2d5b9a1af301ede6a390e394a6e925d1bb5e3b1c))
* **deps:** update rust crate serde_json to v1.0.142 ([85957c0](https://gitlab.com/lx-industries/rmcp-openapi/commit/85957c0c9e9ab1b86ab8492995adc62f457f3dbd))
* **deps:** update rust crate tokio to v1.47.1 ([cdec188](https://gitlab.com/lx-industries/rmcp-openapi/commit/cdec1887141e92a5ff47cd3fbb80fb0d1e8391a6))

## [0.4.0](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.3.2...v0.4.0) (2025-07-28)


### Features

* add backwards compatibility for structured content ([b250a1c](https://gitlab.com/lx-industries/rmcp-openapi/commit/b250a1c951a2dceaeda9ba2d165e78f5094605cd)), closes [#37](https://gitlab.com/lx-industries/rmcp-openapi/issues/37)
* add output schema support for MCP tools ([3f7a5f6](https://gitlab.com/lx-industries/rmcp-openapi/commit/3f7a5f6a30da111eeaaee77497fdccc44aff9b65)), closes [#316](https://gitlab.com/lx-industries/rmcp-openapi/issues/316)
* add parameter value validation against schema with structured error details ([782eacc](https://gitlab.com/lx-industries/rmcp-openapi/commit/782eacc3926e61c025a25cf238e6ae72f1f083a8)), closes [#30](https://gitlab.com/lx-industries/rmcp-openapi/issues/30)
* add support for OpenAPI property names with special characters ([b0299b5](https://gitlab.com/lx-industries/rmcp-openapi/commit/b0299b5b280db29f88e024d941ad988a1dab9c81)), closes [#21](https://gitlab.com/lx-industries/rmcp-openapi/issues/21)
* add title support via ToolAnnotations for OpenAPI operations ([b3febe1](https://gitlab.com/lx-industries/rmcp-openapi/commit/b3febe1b4c03025f640e8eeb5d8186f6c419d953)), closes [#26](https://gitlab.com/lx-industries/rmcp-openapi/issues/26)
* add tool name suggestions to ToolNotFound error ([bc00d26](https://gitlab.com/lx-industries/rmcp-openapi/commit/bc00d26676dad1672d228740656809257bf366f3))
* add validation for unknown tool parameters with 'did you mean' suggestions ([e784e32](https://gitlab.com/lx-industries/rmcp-openapi/commit/e784e32d3d878c86f6165e72bb636784641a0cda)), closes [#24](https://gitlab.com/lx-industries/rmcp-openapi/issues/24)
* implement multiple validation errors with ValidationConstraint enum for better LLM usability ([66edd9c](https://gitlab.com/lx-industries/rmcp-openapi/commit/66edd9c9e2d6deab1d7172cc7707fc314e82a4d4)), closes [#35](https://gitlab.com/lx-industries/rmcp-openapi/issues/35)
* improve error handling with separate validation and execution error types ([76db059](https://gitlab.com/lx-industries/rmcp-openapi/commit/76db05927b4046712d4588b794256045f0c6b8ab)), closes [#36](https://gitlab.com/lx-industries/rmcp-openapi/issues/36)
* refactor error typing for better structuredContent support in error cases ([324c89b](https://gitlab.com/lx-industries/rmcp-openapi/commit/324c89b9292a09d30ebba90136ca306249a93ff0)), closes [#28](https://gitlab.com/lx-industries/rmcp-openapi/issues/28)
* return tool errors as structuredContent when outputSchema is defined ([4e212af](https://gitlab.com/lx-industries/rmcp-openapi/commit/4e212af5d411ae3927432811d208adaa80136d46)), closes [#27](https://gitlab.com/lx-industries/rmcp-openapi/issues/27)


### Bug Fixes

* update validation test to use actual Tool struct ([b2ed17f](https://gitlab.com/lx-industries/rmcp-openapi/commit/b2ed17f838fe2e47ded2531b728d1a8bf56a28c1))


### Miscellaneous Chores

* **deps:** update node.js to 079b6a6 ([abc6e1d](https://gitlab.com/lx-industries/rmcp-openapi/commit/abc6e1d24fd04f35771514e370c8a6c94766bf2f))
* **deps:** update node.js to 37ff334 ([7b88ef6](https://gitlab.com/lx-industries/rmcp-openapi/commit/7b88ef65573a10d9fac8d80367d8e2c9bb51bfda))
* **deps:** update node.js to e515259 ([fd64794](https://gitlab.com/lx-industries/rmcp-openapi/commit/fd6479477e8b131b65aff6dd690a62809d8b3684))
* **deps:** update rust crate tokio to v1.47.0 ([becc26c](https://gitlab.com/lx-industries/rmcp-openapi/commit/becc26c0d78e2b4e8a032cb524d4f5a7f83601d6))
* **deps:** update rust:1.88.0 docker digest to a5c5c4b ([f15d1ca](https://gitlab.com/lx-industries/rmcp-openapi/commit/f15d1ca70c1e89b3eab5d609c17ac4d6577c450b))
* **deps:** update rust:1.88.0 docker digest to af306cf ([0aa5e34](https://gitlab.com/lx-industries/rmcp-openapi/commit/0aa5e3476dfe1015be1c36687b2495541d586fbb))
* **deps:** update rust:1.88.0 docker digest to d8fb475 ([4f911fb](https://gitlab.com/lx-industries/rmcp-openapi/commit/4f911fbb6e6ce50425a23d16504f4869519245fa))

## [0.3.2](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.3.1...v0.3.2) (2025-07-21)


### Miscellaneous Chores

* **deps:** update rust crate serde_json to v1.0.141 ([ab8b6b6](https://gitlab.com/lx-industries/rmcp-openapi/commit/ab8b6b6a95b2070d20388750d1da13662c14c705))

## [0.3.1](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.3.0...v0.3.1) (2025-07-17)


### Miscellaneous Chores

* **deps:** update node.js to v22.17.1 ([d30bdb7](https://gitlab.com/lx-industries/rmcp-openapi/commit/d30bdb71e8019312efb86c860ae1dd850b090e53))
* **deps:** update rust crate rmcp to 0.3.0 ([9b2ad47](https://gitlab.com/lx-industries/rmcp-openapi/commit/9b2ad47cf6ac79c5005d5fb85ab52896756ac5b9))

## [0.3.0](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.2.1...v0.3.0) (2025-07-14)


### Features

* extract actual object properties from request body schema ([f50931b](https://gitlab.com/lx-industries/rmcp-openapi/commit/f50931b175c99afe78d840323357e03a6fb1df16)), closes [#14](https://gitlab.com/lx-industries/rmcp-openapi/issues/14)
* implement $ref resolution for request body schemas ([53f7434](https://gitlab.com/lx-industries/rmcp-openapi/commit/53f7434871517e5d6e2442c6880376dbb1f07cc8)), closes [#18](https://gitlab.com/lx-industries/rmcp-openapi/issues/18)


### Bug Fixes

* avoid reloading OpenAPI spec for each client connection ([2431b06](https://gitlab.com/lx-industries/rmcp-openapi/commit/2431b0668dcb900d2a9bb176aee4e4ef25d21e2a)), closes [#15](https://gitlab.com/lx-industries/rmcp-openapi/issues/15)


### Miscellaneous Chores

* **deps:** update rust crate oas3 to 0.17.0 ([a4a9bf6](https://gitlab.com/lx-industries/rmcp-openapi/commit/a4a9bf6d1487d28d98ad95fea59d68a5c0d7b3f3))

## [0.2.1](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.2.0...v0.2.1) (2025-07-11)


### Miscellaneous Chores

* **deps:** update node.js to 2c3f34d ([795f161](https://gitlab.com/lx-industries/rmcp-openapi/commit/795f161c74d85efc880ff41c9f817c42920903db))
* **deps:** update node.js to v22 ([5d5784a](https://gitlab.com/lx-industries/rmcp-openapi/commit/5d5784a14407080af2bf020277a72594e6d3a600))
* **deps:** update rust crate reqwest to 0.12 ([6624c46](https://gitlab.com/lx-industries/rmcp-openapi/commit/6624c467ea67bb5e2fea3c29315156a80a9296fb))
* **deps:** update rust crate thiserror to v2 ([d165d6d](https://gitlab.com/lx-industries/rmcp-openapi/commit/d165d6dd48d61c594f36399cd4e93bbd5d6f1515))
* **deps:** update rust:1.88.0 docker digest to 5771a3c ([7550855](https://gitlab.com/lx-industries/rmcp-openapi/commit/755085502830f7e9dd171c3481cded65f7b5e7f5))

## [0.2.0](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.1.0...v0.2.0) (2025-07-10)


### Features

* implement standalone OpenAPI MCP server binary ([82e643c](https://gitlab.com/lx-industries/rmcp-openapi/commit/82e643ccfbe0b39e19084e0f020d7941cfef9c4d))

## [0.1.0](https://gitlab.com/lx-industries/rmcp-openapi/compare/v0.0.0...v0.1.0) (2025-07-09)


### Features

* implement core MCP server with OpenAPI tool generation ([c375c23](https://gitlab.com/lx-industries/rmcp-openapi/commit/c375c2333a77ce2e877a848034f00daf5897f1d4))
