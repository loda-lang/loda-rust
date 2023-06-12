; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
mov $83,113
lps $80
  mov $0,$$81
  mov $2,$$83
  mov $2,42
  f31 $0,102180 ; Flood fill at every pixel along the border, connectivity-4.
  mov $$82,$0
  add $81,100
  add $82,100
  add $83,102
lpe

; template: "84db8fc4-1.asm"
; mutate: ReplaceSourceWithHistogram
; mutate: IncrementTargetValueWhereTypeIsDirect
