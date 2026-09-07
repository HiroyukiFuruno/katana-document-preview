## ADDED Requirements

### Requirement: KDVとKRRは一つのV8 runtimeを解決しなければならない
KDV v0.5.6はdirect `v8 =152.2.0`とpublic KRR 0.4.19を一つのregistry V8 packageへ解決しなければならない（MUST）。path/git override、異なるV8 version、同一consumer binaryへの二重linkを残してはならない（MUST NOT）。

#### Scenario: local KDV graphを検証する
- **WHEN** release gateがlocked KDV dependency graphを検証する
- **THEN** `cargo tree -d`はV8 duplicate rootを含まず、`cargo tree -i v8`はKDVとKRRを含む唯一の`v8 152.2.0`を示し、KDV public API consumer link testが成功する

### Requirement: 公開registry artifactをfresh consumerで検証しなければならない
KDV publish後は一時consumerがexact published KDVだけをcrates.ioからresolveし、path/git overrideなしでlinkしなければならない（MUST）。

#### Scenario: publish済みKDVをconsumerがbuildする
- **WHEN** crates.ioでKDV patch releaseが可視になった後にrelease scriptが実行される
- **THEN** fresh lockfileのmetadataはregistry KDVのみを示し、consumer buildとV8 duplicate checkが成功する
