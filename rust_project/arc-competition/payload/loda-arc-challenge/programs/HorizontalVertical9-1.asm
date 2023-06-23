; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
mov $83,113
lps $80
  mov $0,$$81
  f31 $0,102180 ; Flood fill at every pixel along the border, connectivity-4.
  mov $2,42
  mov $1,1
  mov $$82,$0
  add $81,100
  add $82,100
  add $83,100
  add $84,100
lpe

; template: "84db8fc4-1.asm"
; mutate: CallMostPopularProgram, no change
; mutate: SwapRows
; mutate: InsertLineWithHistogram
; mutate: CallRecentProgram, no change
; mutate: ReplaceLineWithHistogram
; mutate: CallRecentProgram, no change
; mutate: SetSourceToConstant
