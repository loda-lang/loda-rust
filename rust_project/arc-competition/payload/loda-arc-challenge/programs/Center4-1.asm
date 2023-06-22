; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
mov $83,104
lps $80
  mov $0,$$81
  mov $1,$$83
  mov $2,42
  f31 $0,102180 ; Flood fill at every pixel along the border, connectivity-4.
  mov $$82,$0
  add $81,100
  add $82,100
lpe

; template: "7b6016b9-1.asm"
; mutate: DecrementSourceValueWhereTypeIsConstant
; mutate: ReplaceLineWithHistogram
; mutate: CallLeastPopularProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: ToggleEnabled
