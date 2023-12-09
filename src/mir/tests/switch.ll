define i1 @switch() {
  %_0 = alloca i1, align 1
  %_1 = alloca i32, align 4
  br label %bb0

bb0:                                              ; preds = %0
  store i32 1, ptr %_1, align 4
  %1 = load i32, ptr %_1, align 4
  switch i32 %1, label %bb1 [
    i32 3, label %bb2
  ]

bb1:                                              ; preds = %bb0
  store i1 false, ptr %_0, align 1
  br label %bb3

bb2:                                              ; preds = %bb0
  store i1 true, ptr %_0, align 1
  br label %bb3

bb3:                                              ; preds = %bb2, %bb1
  %2 = load i1, ptr %_0, align 1
  ret i1 %2
}
