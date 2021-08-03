; A327139: a(n) = n-th number k such that cos(2k) > cos(2k+2) < cos(2k+4).
; 1,4,7,10,13,16,19,23,26,29,32,35,38,41,45,48,51,54,57,60,63,67,70,73,76,79,82,85,89,92,95,98,101,104,107,111,114,117,120,123,snip,324,327,330,MISMATCH
; 106 correct terms.

seq $0,127451 ; Beatty sequence for n/(1 - e^Pi + Pi^e), complement of A127450.
trn $0,2
mov $1,$0
