; A319440: Squares of non-palindromic number.
; 100,144,169,196,225,256,289,324,361,400,441,529,576,625,676,729,784,841,900,961,1024,1156,1225,1296,1369,1444,1521,1600,1681,1764,1849,2025,2116,2209,2304,2401,2500,2601,2704,2809,snip,9409,9604,10000,MISMATCH
; 82 correct terms.

cal $0,139704 ; Nearly palindromic numbers: non-palindromes that can be made palindromic by inserting an extra digit.
pow $0,2
mov $1,$0
