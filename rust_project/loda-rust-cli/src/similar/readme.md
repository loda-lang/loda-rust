# Identify similar programs

Are there patterns in [loda-programs](https://github.com/loda-lang/loda-programs)?

This code generates csv files with a rough estimate of what programs are similar to each other. Only the instructions have been considered. The parameters have not been considered.

The algorithm used for clustering similar programs is [MinHash](https://en.wikipedia.org/wiki/MinHash) or [locality sensitive hashing](https://en.wikipedia.org/wiki/Locality-sensitive_hashing).


# Very similar programs

```csv
program id;overlap
7522;25
30431;25
39949;25
45437;25
45471;25
45473;25
61237;25
61239;25
61241;25
61242;25
62800;25
73521;25
73523;25
88955;25
91968;25
92074;25
92168;25
93191;25
93350;25
93359;25
93838;25
94407;25
94657;25
95995;25
100201;25
```

In the above csv file, it seems that these are highly similar: [A007522](https://oeis.org/A007522), [A030431](https://oeis.org/A030431), [A039949](https://oeis.org/A039949). There seems to be a pattern here.

A pattern can serve as a template for finding new programs.


# Less similar programs

LODA programs that are unique, are also interesting.

It may be the beginning of a new useful pattern, that can be applied over and over.

```csv
program id;overlap
276865;26
188068;19
97538;18
5384;17
96489;17
140461;17
142924;17
290235;17
336298;17
59302;16
122439;16
140649;16
262518;16
2145;15
4139;15
6198;15
7645;15
13637;15
22415;15
25164;15
28430;15
36242;15
36243;15
36244;15
45410;15
```

In the above csv file, the `overlap` column is less than 20, and the referred programs are somewhat similar, but with differencies.

