define void @test() {
  %_1 = alloca i1, align 1
  %_2 = alloca i32, align 4
  br label %bb0

bb0:                                              ; preds = %0
  br label %bb1

bb1:                                              ; preds = %bb0
  ret void
}
