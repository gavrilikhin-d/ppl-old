define i32 @return_value() {
  %_0 = alloca i32, align 4
  br label %bb0

bb0:                                              ; preds = %0
  %1 = load i32, ptr %_0, align 4
  ret i32 %1
}
