def h: ["0","1","2","3","4","5","6","7","8","9","a","b","c","d","e","f"][.];
def hex4: . as $v
  | ((($v/4096)|floor)|h) + ((((($v/256)|floor)%16))|h)
  + ((((($v/16)|floor)%16))|h) + (($v%16)|h);
def grease: [2570,6682,10794,14906,19018,23130,27242,31354,35466,39578,43690,47802,51914,56026,60138,64250];
def nogrease: map(select(. as $x | (grease | index($x)) == null));
.tls as $t
| ($t.cipher_suites | nogrease) as $c
| ([$t.extensions[].codepoint] | nogrease) as $e
| ($t.signature_algorithms | nogrease) as $s
| ($e | map(select(. != 0 and . != 16)) | sort) as $esorted
| (if ($s|length) == 0 then "" else "_" + ($s | map(hex4) | join(",")) end) as $sig
| {
    ciphers_sorted: ($c | sort | map(hex4) | join(",")),
    extensions_sorted: (($esorted | map(hex4) | join(",")) + $sig),
    ciphers_original: ($c | map(hex4) | join(",")),
    extensions_original: (($e | map(hex4) | join(",")) + $sig),
    ncipher: ($c|length), next: ($e|length),
    sni: (if ([$t.extensions[].codepoint] | index(0)) then "d" else "i" end),
    alpn: ($t.alpn[0] // "")
  }
