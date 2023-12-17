; ModuleID = 'test'
source_filename = "test"

define i32 @test() {
  %_0 = alloca i32, align 4
  br label %bb0

bb0:                                              ; preds = %0
  %1 = call i32 @id(i32 1)
  store i32 %1, ptr %_0, align 4
  %2 = load i32, ptr %_0, align 4
  ret i32 %2
}

define i32 @id(i32 %0) {
  %_0 = alloca i32, align 4
  %_1 = alloca i32, align 4
  store i32 %0, ptr %_1, align 4
  br label %bb0

bb0:                                              ; preds = %1
  %2 = load i32, ptr %_1, align 4
  store i32 %2, ptr %_0, align 4
  %3 = load i32, ptr %_0, align 4
  ret i32 %3
}

!llvm.module.flags = !{!0}

!0 = !{i32 2, !"Debug Info Version", i32 3}
