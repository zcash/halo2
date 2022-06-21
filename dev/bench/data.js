window.BENCHMARK_DATA = {
  "lastUpdate": 1655782013477,
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
          "id": "c9fc4c6720229bdc4d1fa44785219125211b2bf9",
          "message": "Merge pull request #601 from zcash/book/update-comparison\n\n[book] Add `U` to `BCMS` comparison table",
          "timestamp": "2022-06-21T03:17:58+01:00",
          "tree_id": "1c149b414ada4bac26354d72223895c60f0a2164",
          "url": "https://github.com/zcash/halo2/commit/c9fc4c6720229bdc4d1fa44785219125211b2bf9"
        },
        "date": 1655782005490,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 91524007,
            "range": "± 2031110",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3996377,
            "range": "± 163578",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 209601514,
            "range": "± 3543319",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 5913482,
            "range": "± 93671",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 289748181,
            "range": "± 4980724",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 7185340,
            "range": "± 145393",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 46176,
            "range": "± 483",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 163372,
            "range": "± 3259",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 177465,
            "range": "± 4739",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 300364,
            "range": "± 2398",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 300197,
            "range": "± 1184",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 173516,
            "range": "± 799",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 188112,
            "range": "± 1611",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 304027,
            "range": "± 1121",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 303962,
            "range": "± 890",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 360387,
            "range": "± 3508",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 368965,
            "range": "± 5623",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 479830,
            "range": "± 7316",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 490407,
            "range": "± 4744",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3584090,
            "range": "± 1268",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 7143254,
            "range": "± 19471",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 13004126,
            "range": "± 205070",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 27989656,
            "range": "± 118611",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 52373795,
            "range": "± 241070",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 97134800,
            "range": "± 955252",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 8438,
            "range": "± 161",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 10063,
            "range": "± 1651",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 18739,
            "range": "± 176",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 23489,
            "range": "± 369",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 33444,
            "range": "± 608",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 55456,
            "range": "± 2260",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 115043,
            "range": "± 10195",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 228705,
            "range": "± 11885",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 471052,
            "range": "± 43468",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 1007966,
            "range": "± 57076",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 2127456,
            "range": "± 71997",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 4606162,
            "range": "± 134788",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 9921626,
            "range": "± 188663",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 21545215,
            "range": "± 539126",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 46619946,
            "range": "± 1363387",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 103961234,
            "range": "± 7264001",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 34091,
            "range": "± 217",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 33805,
            "range": "± 581",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 179876212,
            "range": "± 1305740",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 395864180,
            "range": "± 2425651",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 851178244,
            "range": "± 3331752",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1825295530,
            "range": "± 8396172",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3907495330,
            "range": "± 13092653",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 8359633387,
            "range": "± 36902007",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 17789451418,
            "range": "± 111256662",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 37729721785,
            "range": "± 153097214",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 79776906825,
            "range": "± 231319509",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 113169751,
            "range": "± 2215769",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 194816940,
            "range": "± 1164016",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 351185303,
            "range": "± 951291",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 641493800,
            "range": "± 22285001",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1198053687,
            "range": "± 24327343",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2268711702,
            "range": "± 25793448",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4297013960,
            "range": "± 22014808",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 8312360915,
            "range": "± 140626579",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 16047461382,
            "range": "± 113693145",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5892222,
            "range": "± 104599",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 9397540,
            "range": "± 206270",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 15678287,
            "range": "± 281094",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 26647012,
            "range": "± 733676",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 44739731,
            "range": "± 334791",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 80651062,
            "range": "± 660890",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 143144831,
            "range": "± 1844053",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 264354948,
            "range": "± 5310958",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 480996305,
            "range": "± 6454120",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}