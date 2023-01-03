; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
lps $80
  mov $0,$$81
  mov $5,$0
  f11 $5,101240 ; Number of unique colors in image.
  sub $5,0
  mov $1,$0
  f11 $1,101000 ; Get width of image
  mov $2,$0
  f11 $2,101001 ; Get height of image
  mul $1,$5
  mul $2,$5
  f31 $0,101200 ; Resize image to size width x height
  mov $$82,$0
  add $81,10
  add $82,10
lpe

; template: "b91ae062-1.asm"
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: ReplaceLineWithHistogram
; mutate: CallMediumPopularProgram, no change
; mutate: CallRecentProgram, no change
; mutate: DecrementSourceValueWhereTypeIsConstant
