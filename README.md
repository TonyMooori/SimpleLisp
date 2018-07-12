# SimpleLisp
[mal](https://github.com/kanaka/mal)というLispのRustによる実装．  
名前はSimepleLispだが実装はシンプルではない．
はじめてのちゃんとしたLispインタプリタの実装は失敗に終わった．

# できること
malの仕様のmeta, with-meta以外はだいたいできる

# できないこと
STEP1～6ぐらいの実装の際にテストケースを見てなかったのでところどころ仕様が違う．
そのためmalで実装されたmalを実行できない．
あとmeta, with-metaは最後の最後で言われても困るという感じだったので実装していない．
