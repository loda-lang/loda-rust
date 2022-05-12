; 0,3,12,28,50,78,113,153,201,254,314,380,452,530,615,706,804,907,1017,1134,1256,1385,1520,1661,1809,1963,2123,2290,2463,2642,2827,3019,3216,3421,3631,3848,4071,4300,4536,4778

mov $1,10
pow $0,2
add $2,$0
mul $2,1420
div $2,452
mov $4,10
mod $0,10
mov $0,$2

; template 68079
; mutation: InsertInstructionWithConstant
; mutation: ReplaceTargetWithHistogram
; mutation: ReplaceSourceConstantWithHistogram
; mutation: ReplaceSourceConstantWithHistogram
; mutation: ReplaceInstructionWithHistogram
; mutation: ReplaceSourceConstantWithHistogram
; mutation: SwapAdjacentRows
; mutation: ReplaceInstructionWithHistogram
; mutation: InsertInstructionWithConstant
; mutation: ReplaceTargetWithHistogram
