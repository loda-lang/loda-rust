; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
lps $80
  mov $0,$$81
  mov $5,$0
  mov $1,$0
  mov $2,$0
  f11 $1,101191 ; Image: flip y
  f11 $2,101060 ; Image the 1 most popular colors, sorted by popularity
  f31 $0,101150 ; Image: Overlay another image by using a color as mask
  mov $$82,$0
  add $81,10
  add $82,10
lpe

; template: "496994bd-1.asm"
; mutate: ReplaceLineWithHistogram
; mutate: InsertLineWithHistogram
; mutate: CallRecentProgram, no change
; mutate: SwapAdjacentRows
