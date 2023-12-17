; ModuleID = 'test'
source_filename = "test"

define i32 @call() {
  %_0 = alloca i32, align 4
  br label %bb0

bb0:                                              ; preds = %0
  %1 = call i32 @sum(i32 1, i32 2)
  store i32 %1, ptr %_0, align 4
  %2 = load i32, ptr %_0, align 4
  ret i32 %2
}

declare i32 @sum(i32, i32)

!llvm.module.flags = !{!0}

!0 = !{i32 2, !"Debug Info Version", i32 3}
