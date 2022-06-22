window.BENCHMARK_DATA = {
  "lastUpdate": 1655935198675,
  "repoUrl": "https://github.com/zcash/halo2",
  "entries": {
    "halo2 Benchmark": [
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "str4d",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "745f5d159895bbff70d3086dcfec5ef46aba6981",
          "message": "Merge pull request #607 from zcash/caching\n\nCache values in `VerifyingKey` that can be computed on construction",
          "timestamp": "2022-06-22T21:53:32+01:00",
          "tree_id": "3496128b2a6e5765e9b6fb0b2ea81f6a79374403",
          "url": "https://github.com/zcash/halo2/commit/745f5d159895bbff70d3086dcfec5ef46aba6981"
        },
        "date": 1655935192583,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 81225639,
            "range": "± 332464",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3787828,
            "range": "± 38459",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 184473713,
            "range": "± 1006170",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4752230,
            "range": "± 235489",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 253955344,
            "range": "± 1891950",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5350211,
            "range": "± 139144",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 47006,
            "range": "± 202",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 163374,
            "range": "± 141",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 178875,
            "range": "± 446",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 286851,
            "range": "± 153",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 286829,
            "range": "± 463",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 166816,
            "range": "± 105",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 182347,
            "range": "± 147",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 290339,
            "range": "± 273",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 290258,
            "range": "± 289",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 348894,
            "range": "± 271",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 364392,
            "range": "± 420",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 472028,
            "range": "± 260",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 472263,
            "range": "± 318",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3612392,
            "range": "± 3015",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 6223231,
            "range": "± 34339",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10848134,
            "range": "± 34546",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 23988477,
            "range": "± 160886",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 44011547,
            "range": "± 622958",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 85969978,
            "range": "± 321562",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7622,
            "range": "± 350",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 9044,
            "range": "± 467",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 16753,
            "range": "± 452",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 22481,
            "range": "± 675",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 30844,
            "range": "± 501",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 51360,
            "range": "± 1663",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 105953,
            "range": "± 8762",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 205119,
            "range": "± 11000",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 417121,
            "range": "± 18297",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 875836,
            "range": "± 14411",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1835802,
            "range": "± 26223",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3902702,
            "range": "± 155201",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 8504465,
            "range": "± 833568",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 18432700,
            "range": "± 469368",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 40443289,
            "range": "± 1857513",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 87819040,
            "range": "± 2005343",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 34861,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 35025,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 181808997,
            "range": "± 1362762",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 389796665,
            "range": "± 1677240",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 845237273,
            "range": "± 3415918",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1812193333,
            "range": "± 7227539",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3885022426,
            "range": "± 4383987",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 8303818459,
            "range": "± 12005206",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 17689499715,
            "range": "± 10385054",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 37405294151,
            "range": "± 57998858",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 79397282437,
            "range": "± 182281835",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 112745867,
            "range": "± 715417",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 194464619,
            "range": "± 1384420",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 343286636,
            "range": "± 8129655",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 625714744,
            "range": "± 5746299",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1162800221,
            "range": "± 5305122",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2191886094,
            "range": "± 3767237",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4164158325,
            "range": "± 9081886",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 8080701059,
            "range": "± 225789379",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 15516913698,
            "range": "± 42607179",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 6059814,
            "range": "± 85306",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 9421681,
            "range": "± 110309",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 15408234,
            "range": "± 389499",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 25766706,
            "range": "± 299870",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 43966079,
            "range": "± 482374",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 77976967,
            "range": "± 2003285",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 139679419,
            "range": "± 1148514",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 257027755,
            "range": "± 3951870",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 465070813,
            "range": "± 4594079",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}