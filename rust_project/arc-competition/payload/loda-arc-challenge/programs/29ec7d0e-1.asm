; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
lps $80
  mov $0,$$81
  mov $5,$0
  f11 $5,102141 ; Compare with the pixels above,below,left,right and count how many have the same color as the center.
  mov $6,$80
  f21 $5,101253 ; Convert to a mask image by converting `pixel_color >= threshold_color` to 1 and converting anything else to to 0.
  mov $7,$0
  mov $8,$5
  f21 $7,101231 ; Histogram of image using a mask. Only where the mask is non-zero, are the image pixels added to the histogram.
  mov $8,0
  mov $9,1
  f31 $7,101002 ; Image: get pixel at (x, y)
  mov $10,$0
  mov $7,$11
  f21 $10,102150 ; Fix damaged pixels and recreate simple repeating patterns.
  mov $0,$10
  mov $$82,$0
  add $81,10
  add $82,10
lpe

; template: "0dfd9992-1.asm"
; mutate: CallRecentProgram, no change
; mutate: CallMediumPopularProgram, no change
; mutate: CallRecentProgram, no change
; mutate: ReplaceInstructionWithHistogram, no change
; mutate: CallRecentProgram, no change
; mutate: InsertLineWithHistogram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallMediumPopularProgram, no change
; mutate: CallRecentProgram, no change
; mutate: SetSourceToDirect
; mutate: InsertLineWithHistogram, no change
; mutate: ReplaceSourceWithHistogram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallLeastPopularProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: InsertLineWithHistogram, no change
; mutate: ReplaceSourceWithHistogram, no change
; mutate: CallRecentProgram, no change
; mutate: SwapRegisters
