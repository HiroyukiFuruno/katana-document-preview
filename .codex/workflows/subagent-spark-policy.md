# Subagent / Spark 運用ポリシー

- 許可済みの作業で、分離できる実装・調査は原則 subagent へ移譲する。
- 作業済みactive tasks.mdにはsubagent / Spark証跡または許可済みの
  `delegation-exception:`を残す。
- subagent は `gpt-5.3-codex-spark` / reasoning `medium` を明示する。
- 親モデル継承や `fork_context` 制約を理由に、model / reasoning の明示を省略しない。
- OpenSpec の subagent / Spark 証跡には `file:` を必須化し、`command: multi_agent_v1.spawn_agent` と `close: multi_agent_v1.close_agent` を残す。
- OpenSpec の subagent / Spark 証跡では `command:` を実起動だけに使い、検証コマンドは `verify:` に分ける。
- OpenSpec の subagent / Spark 証跡では `verify:` を必須にし、subagent結果を親が検証した証跡なしに完了扱いしない。
- OpenSpec の `verify:` は、許可済みの `rtk just` / `rtk bash` / `rtk cargo` / `rtk rustfmt` / `rtk env` / `rtk git` / `rtk ./scripts/...` 形式だけを使う。
- 変更された subagent / Spark ハーネス関連ファイルは、OpenSpec証跡または
  許可済み`delegation-exception:`の`file:`で参照する。
- `just check-subagent-harness` の必須コマンドは完全一致で検査し、`|| true` などで失敗を隠せないようにする。
- CIの `just check-subagent-harness` step は `if:` で条件付き実行にしない。
- OpenSpec の作業行で `移譲 / 委譲 / worker / explorer / delegate=subagent` を使った場合も subagent / Spark 証跡の対象にする。
- 作業済みactive `tasks.md` は `<!-- subagent-spark-harness-strict-start -->` 以降の作業行に `証跡:` または `delegation-exception:` を必須化する。
- `delegation-exception:` は `単純な一手作業`、`直列のクリティカルパス`、`書き込み範囲を明示できない`、`ユーザーがsubagent利用を禁止` のいずれかだけを許可する。
- 例外は、単純な一手作業、直列のクリティカルパス、書き込み範囲を明示できない作業、ユーザーがsubagent利用を禁止した場合に限る。
- `必要ならば`、`if required`、`Sparkを使うなら` などの条件付き・裁量扱い表現も禁止する。
- 開始済みactive changeは、`tasks.md` と同階層の`handoff.md`と、その中の
  subagent / Spark証跡または許可済み`delegation-exception:`を必須にする。
- main agent は設計、計画、レビュー、統合判断、ユーザー対話を担当する。
- subagent には分離できる実装・調査を渡し、同じファイルや同じ責務を重ねない。
- 完了済み subagent は速やかに閉じ、新しい並列作業の枠を塞がない。
- OpenSpec の subagent / Spark 関連タスクは `証跡:`、`agent:`、`model:`、`reasoning:`、`file:` を残す。
