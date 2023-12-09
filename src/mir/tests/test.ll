define void @test() {
  %_0 = alloca i64, align 8
  %_1 = alloca i64, align 8
  %_2 = alloca i64, align 8
  br label %bb0

bb0:                                              ; preds = %0
  br label %bb1

bb1:                                              ; preds = %bb0
  ret void
}
