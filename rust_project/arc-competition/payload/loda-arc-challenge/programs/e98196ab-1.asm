; Submitted by Simon Strandgaard
; Program Type: advanced

mov $80,$99
mov $81,100
mov $82,102
lps $80
  mov $0,$$81
  mov $1,$0
  f11 $1,101060 ; Image the 1 most popular colors, sorted by popularity
  mov $2,$0
  f11 $2,101000 ; Get width of image
  div $2,1
  mov $3,$0
  f11 $3,101001 ; Get height of image
  sub $3,1
  div $3,2
  mov $10,$0
  mov $11,$3
  f21 $10,101220 ; get N top rows
  mov $11,$2
  f21 $10,101222 ; get N left columns
  mov $15,$0
  mov $16,$3
  f21 $15,101220 ; get N top rows
  mov $16,$2
  f21 $15,101223 ; get N right columns
  mov $20,$0
  mov $21,$3
  f21 $20,101221 ; get N bottom rows
  mov $21,$2
  f21 $20,101222 ; get N left columns
  mov $25,$0
  mov $26,$3
  f21 $25,101221 ; get N bottom rows
  mov $26,$2
  f21 $25,101223 ; get N right columns
  mov $30,$25
  mov $31,$20
  mov $32,$1
  f31 $30,101150 ; Image: Overlay another image by using a color as mask
  mov $31,$15
  mov $32,$1
  f31 $30,101150 ; Image: Overlay another image by using a color as mask
  mov $32,$10
  mov $32,$1
  f31 $30,101150 ; Image: Overlay another image by using a color as mask
  mov $0,$30
  mov $$82,$0
  add $81,100
  add $82,100
lpe

; template: "a68b268e-1.asm"
; mutate: ToggleEnabled
; mutate: CallRecentProgram, no change
; mutate: IncrementTargetValueWhereTypeIsDirect
; mutate: ReplaceSourceWithHistogram, no change
; mutate: CallLeastPopularProgram, no change
; mutate: CallRecentProgram, no change
; mutate: CallRecentProgram, no change
; mutate: ReplaceSourceConstantWithHistogram, no change
; mutate: CallLeastPopularProgram, no change
; mutate: CallRecentProgram, no change
; mutate: DecrementSourceValueWhereTypeIsConstant
