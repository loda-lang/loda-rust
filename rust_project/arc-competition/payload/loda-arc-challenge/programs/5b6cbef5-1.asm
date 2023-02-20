; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
lps $80
  mov $0,$$81
  mov $22,0
  mov $19,$0
  mov $20,$0
  f11 $20,101000 ; Get width of image
  mov $21,$0
  f11 $21,101000 ; Get width of image
  mov $33,$20
  cmp $33,3
  mov $34,2
  div $34,$34
  mov $33,$21
  cmp $33,3
  mov $34,1
  div $34,2
  mov $0,$20
  pow $0,2
  mov $1,$21
  pow $1,2
  mov $2,0
  f31 $0,101010 ; Create new image with size (x, y) and filled with color z
  mov $11,0
  mov $8,$21
  lps $8
    mov $10,0
    mov $9,$20
    lps $9
      mov $3,$19
      mov $4,$10
      mov $5,$11
      f31 $3,101002 ; Image: get pixel at (x, y)
      cmp $3,$22
      cmp $3,0
      lps $3
        mov $1,$19
        mov $2,$10
        mul $2,$20
        mov $3,$11
        mul $3,$21
        f41 $0,101151 ; Image: Overlay another image at position (x, y)
      lpe
      add $10,1
    lpe
    add $11,1
  lpe
  mov $$82,$0
  add $81,10
  add $82,10
lpe

; template: "007bbfb7-1.asm"
; mutate: CallMediumPopularProgram, no change
; mutate: ReplaceLineWithHistogram
; mutate: InsertLineWithHistogram, no change
; mutate: CallMediumPopularProgram, no change
; mutate: InsertLineWithHistogram, no change
; mutate: ReplaceLineWithHistogram
; mutate: CallRecentProgram, no change
; mutate: SetSourceToConstant
; mutate: ReplaceLineWithHistogram
; mutate: CallRecentProgram, no change
; mutate: IncrementSourceValueWhereTypeIsConstant
; mutate: CallRecentProgram, no change
; mutate: CallMostPopularProgram, no change
; mutate: CallRecentProgram, no change
; mutate: ReplaceLineWithHistogram
; mutate: CallRecentProgram, no change
; mutate: ReplaceLineWithHistogram
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: IncrementSourceValueWhereTypeIsDirect
