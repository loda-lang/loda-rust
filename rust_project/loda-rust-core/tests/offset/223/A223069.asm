; A223069: Number of n X 2 0..3 arrays with successive rows and columns fitting to straight lines with nondecreasing slope, with a single point array taken as having zero slope.
; Submitted by Coleslaw
; 16,150,1080,6627,36552,187000,905440,4206453,18933408,83153850,358250280,1520208679,6373759384,26468569500,109080982800,446806304505,1821267503280,7395000190750,29933239010200,120863093617131,487054223473896,1959657102062400,7874853754399680,31613806764501757,126815552645300032,508394645093717250,2037130271122978440,8159639347358945583,32673301810034530488,130801609522481376100,523544152256294333680,2095227274014876967809,8384173659599897834064,33546826037002377147750,134218711472435638212600

#offset 1

sub $0,1
mov $2,1
mov $10,1
add $0,1
lpb $0
  sub $0,1
  mov $5,0
  mov $6,0
  mov $4,$2
  lpb $4
    trn $4,1
    mov $7,$4
    seq $7,183634 ; Number of (n+1) X 2 0..3 arrays with every 2 x 2 subblock summing to 6.
    mov $9,10
    add $9,$5
    mul $7,$$9
    add $5,1
    add $6,$7
  lpe
  div $6,$2
  mov $9,10
  add $9,$2
  mov $3,$6
  mov $$9,$3
  add $2,1
lpe
mov $0,$3
