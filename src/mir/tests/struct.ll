; ModuleID = 'test'
source_filename = "test"

%Test = type { %Inner }
%Inner = type { i32 }

define i32 @test() {
  %_0 = alloca i32, align 4
  %_1 = alloca %Test, align 8
  br label %bb0

bb0:                                              ; preds = %0
  %1 = getelementptr inbounds %Test, ptr %_1, i32 0, i32 0
  %2 = getelementptr inbounds %Inner, ptr %1, i32 0, i32 0
  store i32 1, ptr %2, align 4
  %3 = load i32, ptr %_0, align 4
  ret i32 %3
}

!llvm.module.flags = !{!0}

!0 = !{i32 2, !"Debug Info Version", i32 3}
