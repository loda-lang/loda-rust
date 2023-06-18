; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
lps $80
  mov $0,$$81
  f11 $0,101230 ; Histogram of image. The most popular to the left, least popular to the right. The top row is the counters. The bottom row is the colors.
  mov $1,100
  f21 $0,101170 ; Image: Rotate by x * 90 degrees
  mov $1,1
  f21 $0,101226 ; remove N left columns
  mov $1,1
  f21 $0,101221 ; get N bottom rows
  mov $$82,$0
  add $81,100
  add $82,100
lpe

; template: "Order8-1.asm"
; mutate: SetSourceToConstant, no change
; mutate: CallLeastPopularProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallLeastPopularProgram, no change
; mutate: CallRecentProgram, no change
; mutate: ReplaceInstructionWithHistogram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallLeastPopularProgram, no change
; mutate: ReplaceSourceConstantWithHistogram
