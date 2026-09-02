## ADDED Requirements

### Requirement: 正当なOOXML ZIP variantを受理しなければならない
KDVのOffice preflightは通常local header、data descriptor、ZIP64を含む正当なDOCX/XLSX/PPTX packageを中央directory検証後に受理しなければならない（MUST）。

#### Scenario: Data descriptorを持つOffice packageを開く
- **WHEN** entry sizeとCRCがlocal headerではなくdata descriptorに記録された正当なOffice packageを開く
- **THEN** KDVはlocal headerだけを理由にInvalidArchiveを返さず、中央directoryとpayload検証を継続する

#### Scenario: KatanAで再現した全entry data descriptor DOCXを開く
- **WHEN** SHA-256 `a1b7e22021218d314bc2d90c526d6d682981828b67cef6e61d8cb2a71ef5742a` の20 entry DOCXを開く
- **THEN** 全local headerのbit 3とCRC/size `0/0/0`を受理し、`word/document.xml`のcentral-directory size `1383/4907`を検証した上でworkerが非空frameを生成する

### Requirement: ZIP安全性検査を弱めてはならない
互換variantを受理する場合も、KDVは重複entry、破損payload、path traversal、active content、external resource、entry/展開量/圧縮率limitを拒否しなければならない（MUST）。

#### Scenario: Data descriptorを装った破損packageを開く
- **WHEN** 中央directory、CRC、entry名または展開量が安全契約を満たさない
- **THEN** KDVはtyped archiveまたはresource-limit errorを返し、workerを起動しない
