window.BENCHMARK_DATA = {
  "lastUpdate": 1679445673221,
  "repoUrl": "https://github.com/zcash/halo2",
  "entries": {
    "halo2 Benchmark": [
      {
        "commit": {
          "author": {
            "email": "ewillbefull@gmail.com",
            "name": "ebfull",
            "username": "ebfull"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "81c7326bf2538c855ee73f12c7c2b336b1a85d16",
          "message": "Merge pull request #757 from zcash/non-exhaustive-errors\n\nMake `Error` a non_exhaustive type.",
          "timestamp": "2023-03-21T17:37:12-06:00",
          "tree_id": "fe3ebe7c284c783f10512d48d50e894292903e63",
          "url": "https://github.com/zcash/halo2/commit/81c7326bf2538c855ee73f12c7c2b336b1a85d16"
        },
        "date": 1679445665157,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 99010553,
            "range": "± 9950539",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 5330635,
            "range": "± 454222",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 181775064,
            "range": "± 4531202",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 6223390,
            "range": "± 678177",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 248334577,
            "range": "± 6618888",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 7642178,
            "range": "± 765839",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 47034,
            "range": "± 120",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 163034,
            "range": "± 168",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 178208,
            "range": "± 127",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 289444,
            "range": "± 198",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 289575,
            "range": "± 250",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 166500,
            "range": "± 714",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 181603,
            "range": "± 307",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 292972,
            "range": "± 119",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 292882,
            "range": "± 374",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 348286,
            "range": "± 175",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 363524,
            "range": "± 256",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 474644,
            "range": "± 216",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 474867,
            "range": "± 1151",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3446897,
            "range": "± 70304",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 5892114,
            "range": "± 3368",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10400099,
            "range": "± 14485",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 21998734,
            "range": "± 70778",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 40560908,
            "range": "± 170693",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 78669128,
            "range": "± 145810",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7183,
            "range": "± 306",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8485,
            "range": "± 1693",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 15042,
            "range": "± 581",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 18700,
            "range": "± 1118",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 27001,
            "range": "± 3004",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 45374,
            "range": "± 6848",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 103465,
            "range": "± 15751",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 223086,
            "range": "± 41997",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 477530,
            "range": "± 55275",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 899272,
            "range": "± 105950",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1691994,
            "range": "± 111411",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3491865,
            "range": "± 385671",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 8058707,
            "range": "± 648443",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 15650832,
            "range": "± 1812746",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 34150479,
            "range": "± 2721074",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 74291253,
            "range": "± 787057",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 34653,
            "range": "± 93",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 34670,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 174592902,
            "range": "± 7070655",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 372844263,
            "range": "± 892987",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 816266371,
            "range": "± 7357944",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1735989198,
            "range": "± 18618377",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3707380535,
            "range": "± 6860751",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 7917231063,
            "range": "± 8907431",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 16880272044,
            "range": "± 20356198",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 35824378334,
            "range": "± 66035139",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 75790595410,
            "range": "± 46642453",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 115173944,
            "range": "± 3714937",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 207362988,
            "range": "± 4153550",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 356184685,
            "range": "± 7734805",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 623225863,
            "range": "± 16064154",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1126588488,
            "range": "± 16210285",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2125904198,
            "range": "± 16689520",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4033143693,
            "range": "± 8791235",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 7778686135,
            "range": "± 54354088",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 14859126245,
            "range": "± 34922289",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5856483,
            "range": "± 288905",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 9026613,
            "range": "± 567490",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 14411430,
            "range": "± 1331961",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 24333887,
            "range": "± 2770893",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 41877913,
            "range": "± 2737494",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 74988532,
            "range": "± 6538530",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 136018437,
            "range": "± 6955554",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 248009155,
            "range": "± 9498486",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 452015560,
            "range": "± 6690994",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}