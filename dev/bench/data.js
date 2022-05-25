window.BENCHMARK_DATA = {
  "lastUpdate": 1653458767446,
  "repoUrl": "https://github.com/zcash/halo2",
  "entries": {
    "halo2 Benchmark": [
      {
        "commit": {
          "author": {
            "email": "yingtong@z.cash",
            "name": "ying tong",
            "username": "therealyingtong"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "34b2e77a573f115c954832c1f0977cd057aac9d0",
          "message": "Merge pull request #467 from zcash/sha256-tweaks\n\nSHA-256 chip tweaks",
          "timestamp": "2022-05-25T12:51:16+08:00",
          "tree_id": "abf678a068dff24f95b3c633ad03668796ad25fe",
          "url": "https://github.com/zcash/halo2/commit/34b2e77a573f115c954832c1f0977cd057aac9d0"
        },
        "date": 1653458761314,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 93794964,
            "range": "± 6520511",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4437570,
            "range": "± 244198",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 212483915,
            "range": "± 6350569",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 6362074,
            "range": "± 335954",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 294776900,
            "range": "± 10311129",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 7653720,
            "range": "± 589952",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 48694,
            "range": "± 1812",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 184696,
            "range": "± 46472",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 201489,
            "range": "± 23777",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 331978,
            "range": "± 14101",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 331470,
            "range": "± 24477",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 189914,
            "range": "± 28258",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 205545,
            "range": "± 8057",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 336560,
            "range": "± 21303",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 332575,
            "range": "± 15590",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 393425,
            "range": "± 16481",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 402808,
            "range": "± 17573",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 545662,
            "range": "± 58859",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 538996,
            "range": "± 21631",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 4168224,
            "range": "± 195760",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 7289039,
            "range": "± 233088",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 13265746,
            "range": "± 236276",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 28367649,
            "range": "± 747424",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 51120728,
            "range": "± 1093100",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 99485659,
            "range": "± 1898656",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7989,
            "range": "± 1258",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 10782,
            "range": "± 1682",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 18690,
            "range": "± 2706",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 22193,
            "range": "± 6142",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 33789,
            "range": "± 2040",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 59313,
            "range": "± 4746",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 128428,
            "range": "± 17068",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 250488,
            "range": "± 23437",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 536818,
            "range": "± 48579",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 1110603,
            "range": "± 73917",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 2350875,
            "range": "± 122469",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 5123590,
            "range": "± 280214",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 11292884,
            "range": "± 661953",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 23940903,
            "range": "± 2107251",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 51480607,
            "range": "± 2826082",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 110515999,
            "range": "± 5878743",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 38423,
            "range": "± 2711",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 36780,
            "range": "± 5928",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 215124226,
            "range": "± 12210117",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 465286082,
            "range": "± 15073417",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 967615437,
            "range": "± 12202013",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 2098866721,
            "range": "± 26331831",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 4482462031,
            "range": "± 36351386",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 9500301418,
            "range": "± 63431050",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 20256024062,
            "range": "± 119237698",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 43115825707,
            "range": "± 112216981",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 91514816671,
            "range": "± 301938599",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 132820480,
            "range": "± 3534412",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 232966513,
            "range": "± 6713968",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 400282867,
            "range": "± 5027910",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 734880606,
            "range": "± 8864638",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1370610794,
            "range": "± 12538933",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2592512742,
            "range": "± 27580851",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4950987226,
            "range": "± 94519587",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 9535382002,
            "range": "± 138657172",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 18198637242,
            "range": "± 115606428",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 7152211,
            "range": "± 546795",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 10957847,
            "range": "± 565517",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 17636038,
            "range": "± 979349",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 30081355,
            "range": "± 2435720",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 50892651,
            "range": "± 3244628",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 90526293,
            "range": "± 6040201",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 163300813,
            "range": "± 5152112",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 298524809,
            "range": "± 9303224",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 550883770,
            "range": "± 12796797",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}