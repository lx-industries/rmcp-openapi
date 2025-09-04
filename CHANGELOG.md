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
