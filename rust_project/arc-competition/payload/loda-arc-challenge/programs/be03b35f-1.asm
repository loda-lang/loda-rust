; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
mov $83,114
lps $80
  mov $0,$$81
  mov $20,$$83
  mov $21,1
  mov $10,$$81
  mov $11,$21
  f22 $10,102261 ; Split image into 2 rows with same size
  mov $15,$10
  mov $16,$21
  f22 $15,102260 ; Split image into 2 columns with same size
  mov $17,$11
  mov $18,$21
  f22 $17,102260 ; Split image into 2 columns with same size
  mov $0,$20
  mov $1,$15
  mov $2,$18
  mov $3,$16
  mov $4,$16
  f51 $0,101152 ; Z-stack images: Overlay multiple images using a transparency color
  mov $$82,$0
  add $81,100
  add $82,100
  add $83,$1
lpe

; template: "ea9794b1-1.asm"
; mutate: CallLeastPopularProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallLeastPopularProgram, no change
; mutate: CallLeastPopularProgram, no change
; mutate: IncrementSourceValueWhereTypeIsConstant
; mutate: SetSourceToDirect
; mutate: CallLeastPopularProgram, no change
; mutate: CallRecentProgram, no change
; mutate: ReplaceLineWithHistogram
; mutate: CallRecentProgram, no change
; mutate: DecrementSourceValueWhereTypeIsDirect
